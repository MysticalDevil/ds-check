use anyhow::Context;
use serde::Deserialize;
use std::sync::LazyLock;

const BASE_URL: &str = "https://platform.deepseek.com";

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[allow(dead_code)]
pub const USAGE_PROMPT: &str = "PROMPT_TOKEN";
pub const USAGE_PROMPT_CACHE_HIT: &str = "PROMPT_CACHE_HIT_TOKEN";
pub const USAGE_PROMPT_CACHE_MISS: &str = "PROMPT_CACHE_MISS_TOKEN";
pub const USAGE_RESPONSE: &str = "RESPONSE_TOKEN";
pub const USAGE_REQUEST: &str = "REQUEST";

#[derive(Debug, Deserialize)]
pub struct BizResponse<T> {
    pub code: i32,
    pub data: Option<BizData<T>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BizData<T> {
    pub biz_code: i32,
    pub biz_data: T,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct IdProfile {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUserData {
    pub id_profile: IdProfile,
    pub email: String,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Wallet {
    pub currency: String,
    pub balance: String,
    pub token_estimation: String,
}

#[derive(Debug, Deserialize)]
pub struct MonthlyCost {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct UserSummaryData {
    pub normal_wallets: Vec<Wallet>,
    pub bonus_wallets: Vec<Wallet>,
    pub monthly_costs: Vec<MonthlyCost>,
    pub monthly_token_usage: String,
    pub current_token: i64,
    pub monthly_usage: String,
    pub total_available_token_estimation: String,
}

#[derive(Debug, Deserialize)]
pub struct UsageAmountData {
    pub total: Vec<ModelUsage>,
    pub days: Vec<DayUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub usage: Vec<UsageItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageItem {
    #[serde(rename = "type")]
    pub usage_type: String,
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DayUsage {
    pub date: String,
    pub data: Vec<ModelUsage>,
}

#[derive(Debug, Clone)]
pub struct DaySummary {
    pub date: String,
    pub model: String,
    pub prompt_tokens: u64,
    pub cache_hit_tokens: u64,
    pub cache_miss_tokens: u64,
    pub response_tokens: u64,
    pub requests: u64,
    pub cost: f64,
}

async fn api_get<T: for<'de> Deserialize<'de>>(token: &str, path: &str) -> anyhow::Result<T> {
    let url = format!("{}{}", BASE_URL, path);
    let client = &*CLIENT;

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("x-app-version", "20240425.0")
        .send()
        .await
        .context("Network error")?;

    let body: serde_json::Value = resp.json().await.context("Parse response failed")?;

    let biz_resp: BizResponse<T> =
        serde_json::from_value(body).context("Deserialize response failed")?;

    if biz_resp.code != 0 {
        anyhow::bail!("API error: code={}", biz_resp.code);
    }

    biz_resp
        .data
        .ok_or_else(|| anyhow::anyhow!("Empty response data"))
        .map(|d| d.biz_data)
}

pub async fn get_current_user(token: &str) -> anyhow::Result<CurrentUserData> {
    api_get::<CurrentUserData>(token, "/auth-api/v0/users/current").await
}

pub async fn get_user_summary(token: &str) -> anyhow::Result<UserSummaryData> {
    api_get::<UserSummaryData>(token, "/api/v0/users/get_user_summary").await
}

pub async fn get_usage_amount(
    token: &str,
    month: u32,
    year: i32,
) -> anyhow::Result<UsageAmountData> {
    let path = format!("/api/v0/usage/amount?month={}&year={}", month, year);
    api_get::<UsageAmountData>(token, &path).await
}

pub async fn get_usage_cost(
    token: &str,
    month: u32,
    year: i32,
) -> anyhow::Result<Vec<UsageAmountData>> {
    let path = format!("/api/v0/usage/cost?month={}&year={}", month, year);
    api_get::<Vec<UsageAmountData>>(token, &path).await
}

pub fn merge_usage(amount: &UsageAmountData, cost: &[UsageAmountData]) -> Vec<DaySummary> {
    let cost_data = cost.first();
    let mut result: Vec<DaySummary> = Vec::new();

    let models: Vec<&str> = amount.total.iter().map(|m| m.model.as_str()).collect();

    for (day_idx, day) in amount.days.iter().enumerate() {
        for (model_idx, model_usage) in day.data.iter().enumerate() {
            let cache_hit = get_amount(&model_usage.usage, USAGE_PROMPT_CACHE_HIT);
            let cache_miss = get_amount(&model_usage.usage, USAGE_PROMPT_CACHE_MISS);
            let prompt = cache_hit + cache_miss;
            let response = get_amount(&model_usage.usage, USAGE_RESPONSE);
            let reqs = get_amount(&model_usage.usage, USAGE_REQUEST);

            let cost_val: f64 = if let Some(cd) = cost_data {
                cd.days
                    .get(day_idx)
                    .and_then(|d| d.data.get(model_idx))
                    .map(|mu| {
                        mu.usage
                            .iter()
                            .filter(|u| u.usage_type != USAGE_REQUEST)
                            .map(|u| u.amount.parse::<f64>().unwrap_or(0.0))
                            .sum()
                    })
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            if prompt == 0 && cache_hit == 0 && cache_miss == 0 && response == 0 && reqs == 0 {
                continue;
            }

            result.push(DaySummary {
                date: day.date.clone(),
                model: models
                    .get(model_idx)
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                prompt_tokens: prompt,
                cache_hit_tokens: cache_hit,
                cache_miss_tokens: cache_miss,
                response_tokens: response,
                requests: reqs,
                cost: cost_val,
            });
        }
    }

    result
}

fn get_amount(items: &[UsageItem], type_name: &str) -> u64 {
    items
        .iter()
        .find(|u| u.usage_type == type_name)
        .and_then(|u| u.amount.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(usage_type: &str, amount: &str) -> UsageItem {
        UsageItem {
            usage_type: usage_type.to_string(),
            amount: amount.to_string(),
        }
    }

    fn model_usage(model: &str, usage: Vec<UsageItem>) -> ModelUsage {
        ModelUsage {
            model: model.to_string(),
            usage,
        }
    }

    fn day_usage(date: &str, data: Vec<ModelUsage>) -> DayUsage {
        DayUsage {
            date: date.to_string(),
            data,
        }
    }

    #[test]
    fn test_get_amount_found() {
        let items = vec![
            item(USAGE_PROMPT_CACHE_HIT, "100"),
            item(USAGE_RESPONSE, "50"),
        ];
        assert_eq!(get_amount(&items, USAGE_PROMPT_CACHE_HIT), 100);
        assert_eq!(get_amount(&items, USAGE_RESPONSE), 50);
    }

    #[test]
    fn test_get_amount_not_found() {
        let items = vec![item(USAGE_PROMPT_CACHE_HIT, "100")];
        assert_eq!(get_amount(&items, USAGE_REQUEST), 0);
    }

    #[test]
    fn test_get_amount_parse_fail() {
        let items = vec![item(USAGE_PROMPT_CACHE_HIT, "not_a_number")];
        assert_eq!(get_amount(&items, USAGE_PROMPT_CACHE_HIT), 0);
    }

    #[test]
    fn test_merge_usage_basic() {
        let amount = UsageAmountData {
            total: vec![model_usage("test-model", vec![])],
            days: vec![day_usage(
                "2024-01-01",
                vec![model_usage(
                    "test-model",
                    vec![
                        item(USAGE_PROMPT_CACHE_HIT, "100"),
                        item(USAGE_PROMPT_CACHE_MISS, "50"),
                        item(USAGE_RESPONSE, "30"),
                        item(USAGE_REQUEST, "5"),
                    ],
                )],
            )],
        };

        let cost = vec![UsageAmountData {
            total: vec![model_usage("test-model", vec![])],
            days: vec![day_usage(
                "2024-01-01",
                vec![model_usage(
                    "test-model",
                    vec![
                        item(USAGE_PROMPT_CACHE_HIT, "0.01"),
                        item(USAGE_PROMPT_CACHE_MISS, "0.02"),
                        item(USAGE_RESPONSE, "0.03"),
                    ],
                )],
            )],
        }];

        let result = merge_usage(&amount, &cost);
        assert_eq!(result.len(), 1);
        let day = &result[0];
        assert_eq!(day.date, "2024-01-01");
        assert_eq!(day.model, "test-model");
        assert_eq!(day.prompt_tokens, 150); // 100 + 50
        assert_eq!(day.cache_hit_tokens, 100);
        assert_eq!(day.cache_miss_tokens, 50);
        assert_eq!(day.response_tokens, 30);
        assert_eq!(day.requests, 5);
        assert!((day.cost - 0.06).abs() < f64::EPSILON);
    }

    #[test]
    fn test_merge_usage_no_cost() {
        let amount = UsageAmountData {
            total: vec![model_usage("m", vec![])],
            days: vec![day_usage(
                "2024-01-01",
                vec![model_usage(
                    "m",
                    vec![
                        item(USAGE_PROMPT_CACHE_HIT, "10"),
                        item(USAGE_RESPONSE, "5"),
                    ],
                )],
            )],
        };

        let result = merge_usage(&amount, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].cost, 0.0);
        assert_eq!(result[0].prompt_tokens, 10);
    }

    #[test]
    fn test_merge_usage_skips_empty() {
        let amount = UsageAmountData {
            total: vec![model_usage("m", vec![])],
            days: vec![day_usage(
                "2024-01-01",
                vec![model_usage("m", vec![item(USAGE_REQUEST, "0")])],
            )],
        };

        let result = merge_usage(&amount, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_usage_prompt_is_cache_sum() {
        // Verify the fix: prompt_tokens = cache_hit + cache_miss
        let amount = UsageAmountData {
            total: vec![model_usage("m", vec![])],
            days: vec![day_usage(
                "2024-01-01",
                vec![model_usage(
                    "m",
                    vec![
                        item(USAGE_PROMPT, "0"),
                        item(USAGE_PROMPT_CACHE_HIT, "1000"),
                        item(USAGE_PROMPT_CACHE_MISS, "200"),
                    ],
                )],
            )],
        };

        let result = merge_usage(&amount, &[]);
        assert_eq!(result[0].prompt_tokens, 1200);
    }
}
