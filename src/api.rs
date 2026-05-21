use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

const BASE_URL: &str = "https://platform.deepseek.com";
const API_BASE_URL: &str = "https://api.deepseek.com";
const APP_VERSION: &str = "20240425.0";

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub const USAGE_PROMPT_CACHE_HIT: &str = "PROMPT_CACHE_HIT_TOKEN";
pub const USAGE_PROMPT_CACHE_MISS: &str = "PROMPT_CACHE_MISS_TOKEN";
pub const USAGE_RESPONSE: &str = "RESPONSE_TOKEN";
pub const USAGE_REQUEST: &str = "REQUEST";

pub(crate) const USAGE_PROMPT: &str = "PROMPT_TOKEN";

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

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IdProfile {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentUserData {
    pub id_profile: IdProfile,
    pub email: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Wallet {
    pub currency: String,
    pub balance: String,
    pub token_estimation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyCost {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageAmountData {
    pub total: Vec<ModelUsage>,
    pub days: Vec<DayUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub usage: Vec<UsageItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageItem {
    #[serde(rename = "type")]
    pub usage_type: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

async fn api_get<T>(token: &str, path: &str, locale: &crate::i18n::Locale) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    api_get_base(token, path, locale, BASE_URL).await
}

async fn api_get_base<T>(
    token: &str,
    path: &str,
    locale: &crate::i18n::Locale,
    base_url: &str,
) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    // 1. Check cache
    if let Some(cache_path) = crate::cache::cache_path(token, path)
        && let Some(cached) = crate::cache::read_cache::<T>(&cache_path)
    {
        return Ok(cached);
    }

    // 2. Make HTTP request
    let url = format!("{}{}", base_url, path);

    let resp = CLIENT
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("x-app-version", APP_VERSION)
        .send()
        .await
        .context(locale.t("network_error"))?;

    let body: serde_json::Value = resp.json().await.context(locale.t("parse_failed"))?;

    let biz_resp: BizResponse<T> =
        serde_json::from_value(body).context(locale.t("parse_failed"))?;

    if biz_resp.code != 0 {
        if biz_resp.code == 40003 {
            anyhow::bail!("{}\n{}", locale.t("auth_expired"), locale.t("auth_hint"));
        }
        anyhow::bail!("API error: code={}", biz_resp.code);
    }

    let result = biz_resp
        .data
        .ok_or_else(|| anyhow::anyhow!("{}", locale.t("empty_data")))
        .map(|d| d.biz_data)?;

    // 3. Write cache
    if let Some(cache_path) = crate::cache::cache_path(token, path) {
        let _ = crate::cache::write_cache(&cache_path, &result);
    }

    Ok(result)
}

pub async fn get_current_user(
    token: &str,
    locale: &crate::i18n::Locale,
) -> anyhow::Result<CurrentUserData> {
    api_get::<CurrentUserData>(token, "/auth-api/v0/users/current", locale).await
}

pub async fn get_user_summary(
    token: &str,
    locale: &crate::i18n::Locale,
) -> anyhow::Result<UserSummaryData> {
    api_get::<UserSummaryData>(token, "/api/v0/users/get_user_summary", locale).await
}

pub async fn get_usage_amount(
    token: &str,
    month: u32,
    year: i32,
    locale: &crate::i18n::Locale,
) -> anyhow::Result<UsageAmountData> {
    let path = format!("/api/v0/usage/amount?month={}&year={}", month, year);
    api_get::<UsageAmountData>(token, &path, locale).await
}

pub async fn get_usage_cost(
    token: &str,
    month: u32,
    year: i32,
    locale: &crate::i18n::Locale,
) -> anyhow::Result<Vec<UsageAmountData>> {
    let path = format!("/api/v0/usage/cost?month={}&year={}", month, year);
    api_get::<Vec<UsageAmountData>>(token, &path, locale).await
}

// ── Pricing (built-in JSON) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model: String,
    pub input_cache_hit: String,
    pub input_cache_miss: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingData {
    pub currency: String,
    pub unit: String,
    #[serde(default)]
    pub note: String,
    pub models: Vec<ModelPricing>,
}

fn pricing_cache_path() -> Option<std::path::PathBuf> {
    crate::cache::base_dir().map(|p| p.join("pricing.json"))
}

pub fn load_pricing() -> anyhow::Result<PricingData> {
    let path = pricing_cache_path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;
    let json = std::fs::read_to_string(&path)?;
    let data: PricingData = serde_json::from_str(&json)?;
    Ok(data)
}

// ── API Key interface (api.deepseek.com) ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiModel {
    pub id: String,
    pub object: String,
    pub owned_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiModelList {
    pub object: String,
    pub data: Vec<ApiModel>,
}

pub async fn get_models(
    api_key: &str,
    locale: &crate::i18n::Locale,
) -> anyhow::Result<Vec<String>> {
    let resp = CLIENT
        .get(format!("{}/models", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/json")
        .send()
        .await
        .context(locale.t("network_error"))?;

    let list: ApiModelList = resp.json().await.context(locale.t("parse_failed"))?;
    let models: Vec<String> = list.data.into_iter().map(|m| m.id).collect();
    Ok(models)
}

pub fn merge_usage(amount: &UsageAmountData, cost: &[UsageAmountData]) -> Vec<DaySummary> {
    let cost_data = cost.first();
    let mut result: Vec<DaySummary> = Vec::new();

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
                model: model_usage.model.clone(),
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

    #[test]
    fn test_merge_usage_uses_day_model_name() {
        // Regression: model name comes from day.data[].model, not amount.total[]
        let amount = UsageAmountData {
            total: vec![model_usage("wrong-name", vec![])],
            days: vec![day_usage(
                "2024-01-01",
                vec![model_usage(
                    "correct-name",
                    vec![item(USAGE_PROMPT_CACHE_HIT, "100")],
                )],
            )],
        };

        let result = merge_usage(&amount, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].model, "correct-name");
    }

    // ── HTTP mock tests ──────────────────────────────────────

    use crate::i18n::Locale;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn biz_response_json(code: i32, data: Option<serde_json::Value>) -> serde_json::Value {
        let mut resp = serde_json::json!({"code": code});
        if let Some(d) = data {
            resp["data"] = serde_json::json!({
                "biz_code": 0,
                "biz_data": d,
            });
        }
        resp
    }

    #[tokio::test]
    async fn test_api_get_success() {
        let server = MockServer::start().await;
        let user_data = serde_json::json!({
            "id_profile": {"name": "Test", "email": null},
            "email": "test@example.com",
            "currency": "CNY",
        });

        Mock::given(method("GET"))
            .and(path("/auth-api/v0/users/current"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(biz_response_json(0, Some(user_data))),
            )
            .mount(&server)
            .await;

        let result = api_get_base::<CurrentUserData>(
            "fake-token",
            "/auth-api/v0/users/current",
            &Locale::EnUS,
            &server.uri(),
        )
        .await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id_profile.name, "Test");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_api_get_code_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v0/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(biz_response_json(500, None)))
            .mount(&server)
            .await;

        let result = api_get_base::<serde_json::Value>(
            "fake-token",
            "/api/v0/test",
            &Locale::EnUS,
            &server.uri(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("code=500"));
    }

    #[tokio::test]
    async fn test_api_get_auth_expired() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v0/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(biz_response_json(40003, None)))
            .mount(&server)
            .await;

        let result = api_get_base::<serde_json::Value>(
            "fake-token",
            "/api/v0/test",
            &Locale::EnUS,
            &server.uri(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expired") || err.contains("invalid"));
    }

    #[tokio::test]
    async fn test_api_get_empty_data() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v0/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"code": 0})))
            .mount(&server)
            .await;

        let result = api_get_base::<serde_json::Value>(
            "fake-token",
            "/api/v0/test",
            &Locale::EnUS,
            &server.uri(),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty response"));
    }

    #[tokio::test]
    async fn test_api_get_parse_failure_on_empty_body() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v0/test"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let result = api_get_base::<serde_json::Value>(
            "fake-token",
            "/api/v0/test",
            &Locale::EnUS,
            &server.uri(),
        )
        .await;

        // 500 with no body causes JSON parse failure
        assert!(result.is_err());
    }
}
