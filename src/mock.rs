use crate::api;
use chrono::Datelike;

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn days_in_month() -> u32 {
    let today = today();
    let y = today.year();
    let m = today.month();
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
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
    let hit: u64 = sum_usage_type(days, model, "PROMPT_CACHE_HIT_TOKEN");
    let miss: u64 = sum_usage_type(days, model, "PROMPT_CACHE_MISS_TOKEN");
    let response: u64 = sum_usage_type(days, model, "RESPONSE_TOKEN");
    let requests: u64 = sum_usage_type(days, model, "REQUEST");

    model_usage(
        model,
        &[
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", &hit.to_string()),
            ("PROMPT_CACHE_MISS_TOKEN", &miss.to_string()),
            ("RESPONSE_TOKEN", &response.to_string()),
            ("REQUEST", &requests.to_string()),
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

fn calc_cost(usage_type: &str, amount: &str) -> f64 {
    let tokens: f64 = amount.parse().unwrap_or(0.0);
    match usage_type {
        "PROMPT_CACHE_HIT_TOKEN" => tokens * 0.025 / 1_000_000.0,
        "PROMPT_CACHE_MISS_TOKEN" => tokens * 0.55 / 1_000_000.0,
        "RESPONSE_TOKEN" => tokens * 2.19 / 1_000_000.0,
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
                amount: format!("{:.10}", calc_cost(&u.usage_type, &u.amount)),
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
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "0"),
            ("PROMPT_CACHE_MISS_TOKEN", "0"),
            ("RESPONSE_TOKEN", "0"),
            ("REQUEST", "0"),
        ],
    )
}

fn mock_day_tokens(day: u32) -> Vec<(&'static str, &'static str)> {
    match day {
        1 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "1235072"),
            ("PROMPT_CACHE_MISS_TOKEN", "259591"),
            ("RESPONSE_TOKEN", "33927"),
            ("REQUEST", "37"),
        ],
        2 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "45691008"),
            ("PROMPT_CACHE_MISS_TOKEN", "944444"),
            ("RESPONSE_TOKEN", "284599"),
            ("REQUEST", "432"),
        ],
        3 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "12284672"),
            ("PROMPT_CACHE_MISS_TOKEN", "70253"),
            ("RESPONSE_TOKEN", "132405"),
            ("REQUEST", "130"),
        ],
        4 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "21178880"),
            ("PROMPT_CACHE_MISS_TOKEN", "311968"),
            ("RESPONSE_TOKEN", "138126"),
            ("REQUEST", "186"),
        ],
        _ => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "0"),
            ("PROMPT_CACHE_MISS_TOKEN", "0"),
            ("RESPONSE_TOKEN", "0"),
            ("REQUEST", "0"),
        ],
    }
}

fn mock_flash_day_tokens(day: u32) -> Vec<(&'static str, &'static str)> {
    match day {
        1 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "512000"),
            ("PROMPT_CACHE_MISS_TOKEN", "128000"),
            ("RESPONSE_TOKEN", "45000"),
            ("REQUEST", "120"),
        ],
        2 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "2048000"),
            ("PROMPT_CACHE_MISS_TOKEN", "512000"),
            ("RESPONSE_TOKEN", "180000"),
            ("REQUEST", "480"),
        ],
        3 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "1024000"),
            ("PROMPT_CACHE_MISS_TOKEN", "256000"),
            ("RESPONSE_TOKEN", "90000"),
            ("REQUEST", "240"),
        ],
        4 => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "4096000"),
            ("PROMPT_CACHE_MISS_TOKEN", "1024000"),
            ("RESPONSE_TOKEN", "360000"),
            ("REQUEST", "960"),
        ],
        _ => vec![
            ("PROMPT_TOKEN", "0"),
            ("PROMPT_CACHE_HIT_TOKEN", "0"),
            ("PROMPT_CACHE_MISS_TOKEN", "0"),
            ("RESPONSE_TOKEN", "0"),
            ("REQUEST", "0"),
        ],
    }
}


