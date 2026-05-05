use crate::api;
use chrono::Datelike;

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn days_in_month() -> u32 {
    let today = today();
    let days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let m = today.month() as usize;
    if m == 2 && today.year() % 4 == 0 && (today.year() % 100 != 0 || today.year() % 400 == 0) {
        29
    } else {
        days[m - 1]
    }
}

pub fn mock_user_summary() -> api::UserSummaryData {
    api::UserSummaryData {
        normal_wallets: vec![api::Wallet {
            currency: "CNY".into(),
            balance: "121.7582374400000000".into(),
            token_estimation: "40586079".into(),
        }],
        bonus_wallets: vec![],
        monthly_costs: vec![api::MonthlyCost {
            currency: "CNY".into(),
            amount: "10.3028508000000000".into(),
        }],
        monthly_token_usage: "82564945".into(),
        current_token: 10_000_000,
        monthly_usage: "82564945".into(),
        total_available_token_estimation: "40586079".into(),
    }
}

pub fn mock_api_models() -> Vec<String> {
    vec![
        "deepseek-v4-flash".into(),
        "deepseek-v4-pro".into(),
    ]
}

pub fn mock_usage_amount() -> api::UsageAmountData {
    let today = today();
    let days: Vec<api::DayUsage> = (1..=days_in_month())
        .map(|d| api::DayUsage {
            date: format!("{:04}-{:02}-{:02}", today.year(), today.month(), d),
            data: vec![
                model_usage("deepseek-v4-pro", &mock_day_tokens(d)),
                model_usage("deepseek-v4-flash", &mock_flash_day_tokens(d)),
                model_usage("deepseek-chat & deepseek-reasoner", &[]),
            ],
        })
        .collect();

    api::UsageAmountData {
        total: vec![
            model_usage_totals(&days, "deepseek-v4-pro"),
            model_usage_totals(&days, "deepseek-v4-flash"),
            empty_model("deepseek-chat & deepseek-reasoner"),
        ],
        days,
    }
}

fn model_usage_totals(days: &[api::DayUsage], model: &str) -> api::ModelUsage {
    let hit: u64 = sum_usage_type(days, model, api::USAGE_PROMPT_CACHE_HIT);
    let miss: u64 = sum_usage_type(days, model, api::USAGE_PROMPT_CACHE_MISS);
    let response: u64 = sum_usage_type(days, model, api::USAGE_RESPONSE);
    let requests: u64 = sum_usage_type(days, model, api::USAGE_REQUEST);

    model_usage(
        model,
        &[
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, &hit.to_string()),
            (api::USAGE_PROMPT_CACHE_MISS, &miss.to_string()),
            (api::USAGE_RESPONSE, &response.to_string()),
            (api::USAGE_REQUEST, &requests.to_string()),
        ],
    )
}

fn sum_usage_type(days: &[api::DayUsage], model: &str, usage_type: &str) -> u64 {
    days.iter()
        .flat_map(|d| d.data.iter())
        .filter(|m| m.model == model)
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == usage_type)
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum()
}

fn model_token_cost(model: &str, usage_type: &str, amount: &str) -> f64 {
    let tokens: f64 = amount.parse().unwrap_or(0.0);
    let is_flash = model.contains("flash");
    match usage_type {
        api::USAGE_PROMPT_CACHE_HIT => tokens * 0.025 / 1_000_000.0,
        api::USAGE_PROMPT_CACHE_MISS => {
            let rate = if is_flash { 1.0 } else { 3.0 };
            tokens * rate / 1_000_000.0
        }
        api::USAGE_RESPONSE => {
            let rate = if is_flash { 2.0 } else { 6.0 };
            tokens * rate / 1_000_000.0
        }
        _ => 0.0,
    }
}

fn convert_to_cost(model: &api::ModelUsage) -> api::ModelUsage {
    api::ModelUsage {
        model: model.model.clone(),
        usage: model
            .usage
            .iter()
            .map(|u| api::UsageItem {
                usage_type: u.usage_type.clone(),
                amount: format!("{:.10}", model_token_cost(&model.model, &u.usage_type, &u.amount)),
            })
            .collect(),
    }
}

pub fn mock_usage_cost() -> Vec<api::UsageAmountData> {
    let amount = mock_usage_amount();
    let cost_data = api::UsageAmountData {
        total: amount.total.iter().map(convert_to_cost).collect(),
        days: amount
            .days
            .iter()
            .map(|d| api::DayUsage {
                date: d.date.clone(),
                data: d.data.iter().map(convert_to_cost).collect(),
            })
            .collect(),
    };
    vec![cost_data]
}

fn model_usage(model: &str, items: &[(&str, &str)]) -> api::ModelUsage {
    api::ModelUsage {
        model: model.into(),
        usage: items
            .iter()
            .map(|(t, a)| api::UsageItem {
                usage_type: t.to_string(),
                amount: a.to_string(),
            })
            .collect(),
    }
}

fn empty_model(model: &str) -> api::ModelUsage {
    model_usage(
        model,
        &[
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "0"),
            (api::USAGE_PROMPT_CACHE_MISS, "0"),
            (api::USAGE_RESPONSE, "0"),
            (api::USAGE_REQUEST, "0"),
        ],
    )
}

fn mock_day_tokens(day: u32) -> Vec<(&'static str, &'static str)> {
    match day {
        1 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "1235072"),
            (api::USAGE_PROMPT_CACHE_MISS, "259591"),
            (api::USAGE_RESPONSE, "33927"),
            (api::USAGE_REQUEST, "37"),
        ],
        2 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "45691008"),
            (api::USAGE_PROMPT_CACHE_MISS, "944444"),
            (api::USAGE_RESPONSE, "284599"),
            (api::USAGE_REQUEST, "432"),
        ],
        3 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "12284672"),
            (api::USAGE_PROMPT_CACHE_MISS, "70253"),
            (api::USAGE_RESPONSE, "132405"),
            (api::USAGE_REQUEST, "130"),
        ],
        4 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "21178880"),
            (api::USAGE_PROMPT_CACHE_MISS, "311968"),
            (api::USAGE_RESPONSE, "138126"),
            (api::USAGE_REQUEST, "186"),
        ],
        _ => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "0"),
            (api::USAGE_PROMPT_CACHE_MISS, "0"),
            (api::USAGE_RESPONSE, "0"),
            (api::USAGE_REQUEST, "0"),
        ],
    }
}

fn mock_flash_day_tokens(day: u32) -> Vec<(&'static str, &'static str)> {
    match day {
        1 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "512000"),
            (api::USAGE_PROMPT_CACHE_MISS, "128000"),
            (api::USAGE_RESPONSE, "45000"),
            (api::USAGE_REQUEST, "120"),
        ],
        2 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "2048000"),
            (api::USAGE_PROMPT_CACHE_MISS, "512000"),
            (api::USAGE_RESPONSE, "180000"),
            (api::USAGE_REQUEST, "480"),
        ],
        3 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "1024000"),
            (api::USAGE_PROMPT_CACHE_MISS, "256000"),
            (api::USAGE_RESPONSE, "90000"),
            (api::USAGE_REQUEST, "240"),
        ],
        4 => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "4096000"),
            (api::USAGE_PROMPT_CACHE_MISS, "1024000"),
            (api::USAGE_RESPONSE, "360000"),
            (api::USAGE_REQUEST, "960"),
        ],
        _ => vec![
            (api::USAGE_PROMPT, "0"),
            (api::USAGE_PROMPT_CACHE_HIT, "0"),
            (api::USAGE_PROMPT_CACHE_MISS, "0"),
            (api::USAGE_RESPONSE, "0"),
            (api::USAGE_REQUEST, "0"),
        ],
    }
}
