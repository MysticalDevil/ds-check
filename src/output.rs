use crate::api::{DaySummary, UserSummaryData};
use crate::i18n::Locale;

pub fn print_summary(
    summary: &UserSummaryData,
    requests: u64,
    nickname: &str,
    json: bool,
    locale: Locale,
) {
    let balance = summary
        .normal_wallets
        .first()
        .map(|w| (w.balance.as_str(), w.currency.as_str()))
        .unwrap_or(("0", "CNY"));

    let cost = summary
        .monthly_costs
        .first()
        .map(|c| (c.amount.as_str(), c.currency.as_str()))
        .unwrap_or(("0", "CNY"));

    let tokens = &summary.monthly_token_usage;

    if json {
        let output = serde_json::json!({
            "user": nickname,
            "balance": {
                "amount": balance.0.parse::<f64>().unwrap_or(0.0),
                "currency": balance.1,
            },
            "monthly_cost": {
                "amount": cost.0.parse::<f64>().unwrap_or(0.0),
                "currency": cost.1,
            },
            "api_requests": requests,
            "tokens": tokens.parse::<u64>().unwrap_or(0),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return;
    }

    let header = locale.t("header");
    let user_label = locale.t("user");
    let balance_label = locale.t("balance");
    let cost_label = locale.t("monthly_cost");
    let req_label = locale.t("api_requests");
    let token_label = locale.t("tokens");

    println!("============ {} ============", header);
    println!("{:>10}: {}", user_label, nickname);
    println!(
        "{:>10}: {:.2} {}",
        balance_label,
        balance.0.parse::<f64>().unwrap_or(0.0),
        balance.1,
    );
    println!(
        "{:>10}: {:.2} {}",
        cost_label,
        cost.0.parse::<f64>().unwrap_or(0.0),
        cost.1,
    );
    println!("{:>10}: {}", req_label, format_num(requests));
    println!("{:>10}: {}", token_label, format_num(tokens.parse().unwrap_or(0)));
    println!("========================================");
}

pub fn print_usage(
    days: &[DaySummary],
    model_filter: Option<&str>,
    json: bool,
    locale: Locale,
) {
    let filtered: Vec<&DaySummary> = if let Some(model) = model_filter {
        days.iter()
            .filter(|d| model_matches(&d.model, model))
            .collect()
    } else {
        days.iter().collect()
    };

    if filtered.is_empty() {
        println!("{}", locale.t("no_data"));
        return;
    }

    if json {
        let output: Vec<serde_json::Value> = filtered
            .iter()
            .map(|d| {
                serde_json::json!({
                    "date": d.date,
                    "model": d.model,
                    "prompt_tokens": d.prompt_tokens,
                    "cache_hit_tokens": d.cache_hit_tokens,
                    "cache_miss_tokens": d.cache_miss_tokens,
                    "response_tokens": d.response_tokens,
                    "requests": d.requests,
                    "cost": d.cost,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return;
    }

    let model_label = locale.t("model");
    let date_label = locale.t("date");
    let prompt_label = locale.t("prompt_tokens");
    let cache_hit_label = locale.t("cache_hit_tokens");
    let cache_miss_label = locale.t("cache_miss_tokens");
    let resp_label = locale.t("response_tokens");
    let req_label = locale.t("requests");
    let cost_label = locale.t("cost");
    let total_label = locale.t("total");

    println!("{}: {}", model_label, filtered.first().map(|d| d.model.as_str()).unwrap_or("-"));
    println!();
    println!(
        "{:<12} {:>12} {:>12} {:>12} {:>12} {:>8} {:>10}",
        date_label,
        prompt_label,
        cache_hit_label,
        cache_miss_label,
        resp_label,
        req_label,
        cost_label,
    );
    println!("{}", "-".repeat(82));

    let mut total_prompt: u64 = 0;
    let mut total_cache_hit: u64 = 0;
    let mut total_cache_miss: u64 = 0;
    let mut total_response: u64 = 0;
    let mut total_requests: u64 = 0;
    let mut total_cost: f64 = 0.0;

    for day in &filtered {
        println!(
            "{:<12} {:>12} {:>12} {:>12} {:>12} {:>8} {:>9.2}",
            day.date,
            format_num(day.prompt_tokens),
            format_num(day.cache_hit_tokens),
            format_num(day.cache_miss_tokens),
            format_num(day.response_tokens),
            format_num(day.requests),
            day.cost,
        );
        total_prompt += day.prompt_tokens;
        total_cache_hit += day.cache_hit_tokens;
        total_cache_miss += day.cache_miss_tokens;
        total_response += day.response_tokens;
        total_requests += day.requests;
        total_cost += day.cost;
    }

    println!("{}", "-".repeat(82));
    println!(
        "{:<12} {:>12} {:>12} {:>12} {:>12} {:>8} {:>9.2}",
        total_label,
        format_num(total_prompt),
        format_num(total_cache_hit),
        format_num(total_cache_miss),
        format_num(total_response),
        format_num(total_requests),
        total_cost,
    );
}

fn model_matches(actual: &str, filter: &str) -> bool {
    let a = actual.to_lowercase();
    let f = filter.to_lowercase();
    a == f || a.contains(&f) || f.contains(&a)
}

fn format_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}
