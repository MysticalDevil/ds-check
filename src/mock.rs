use crate::api;
use chrono::Datelike;

fn now_month() -> u32 {
    chrono::Local::now().month()
}
fn now_year() -> i32 {
    chrono::Local::now().year()
}

pub fn _mock_current_user() -> api::CurrentUserData {
    api::CurrentUserData {
        id_profile: api::IdProfile {
            name: "MockUser".into(),
            email: Some("mock@example.com".into()),
        },
        email: "mock@example.com".into(),
        currency: "CNY".into(),
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
    let days: Vec<api::DayUsage> = (1..=now_month_day_count())
        .map(|d| api::DayUsage {
            date: format!("{:04}-{:02}-{:02}", now_year(), now_month(), d),
            data: vec![
                model_usage("deepseek-v4-pro", &mock_day_tokens(d)),
                model_usage("deepseek-v4-flash", &[]),
                model_usage("deepseek-chat & deepseek-reasoner", &[]),
            ],
        })
        .collect();

    let total_prompt_cache_hit: u64 = days
        .iter()
        .flat_map(|d| d.data.iter())
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == "PROMPT_CACHE_HIT_TOKEN")
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum();
    let total_prompt_cache_miss: u64 = days
        .iter()
        .flat_map(|d| d.data.iter())
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == "PROMPT_CACHE_MISS_TOKEN")
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum();
    let total_response: u64 = days
        .iter()
        .flat_map(|d| d.data.iter())
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == "RESPONSE_TOKEN")
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum();
    let total_requests: u64 = days
        .iter()
        .flat_map(|d| d.data.iter())
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == "REQUEST")
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum();

    api::UsageAmountData {
        total: vec![
            model_usage(
                "deepseek-v4-pro",
                &[
                    ("PROMPT_TOKEN", "0"),
                    (
                        "PROMPT_CACHE_HIT_TOKEN",
                        &total_prompt_cache_hit.to_string(),
                    ),
                    (
                        "PROMPT_CACHE_MISS_TOKEN",
                        &total_prompt_cache_miss.to_string(),
                    ),
                    ("RESPONSE_TOKEN", &total_response.to_string()),
                    ("REQUEST", &total_requests.to_string()),
                ],
            ),
            empty_model("deepseek-v4-flash"),
            empty_model("deepseek-chat & deepseek-reasoner"),
        ],
        days,
    }
}

pub fn mock_usage_cost() -> Vec<api::UsageAmountData> {
    let amount = mock_usage_amount();
    let cost_data = api::UsageAmountData {
        total: amount
            .total
            .iter()
            .map(|m| api::ModelUsage {
                model: m.model.clone(),
                usage: m
                    .usage
                    .iter()
                    .map(|u| {
                        let tokens: f64 = u.amount.parse().unwrap_or(0.0);
                        let cost = match u.usage_type.as_str() {
                            "PROMPT_CACHE_HIT_TOKEN" => tokens * 0.025 / 1_000_000.0,
                            "PROMPT_CACHE_MISS_TOKEN" => tokens * 0.55 / 1_000_000.0,
                            "RESPONSE_TOKEN" => tokens * 2.19 / 1_000_000.0,
                            _ => 0.0,
                        };
                        api::UsageItem {
                            usage_type: u.usage_type.clone(),
                            amount: format!("{:.10}", cost),
                        }
                    })
                    .collect(),
            })
            .collect(),
        days: amount
            .days
            .iter()
            .map(|d| api::DayUsage {
                date: d.date.clone(),
                data: d
                    .data
                    .iter()
                    .map(|m| api::ModelUsage {
                        model: m.model.clone(),
                        usage: m
                            .usage
                            .iter()
                            .map(|u| {
                                let tokens: f64 = u.amount.parse().unwrap_or(0.0);
                                let cost = match u.usage_type.as_str() {
                                    "PROMPT_CACHE_HIT_TOKEN" => tokens * 0.025 / 1_000_000.0,
                                    "PROMPT_CACHE_MISS_TOKEN" => tokens * 0.55 / 1_000_000.0,
                                    "RESPONSE_TOKEN" => tokens * 2.19 / 1_000_000.0,
                                    _ => 0.0,
                                };
                                api::UsageItem {
                                    usage_type: u.usage_type.clone(),
                                    amount: format!("{:.10}", cost),
                                }
                            })
                            .collect(),
                    })
                    .collect(),
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

fn now_month_day_count() -> u32 {
    let y = now_year();
    let m = now_month();
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
