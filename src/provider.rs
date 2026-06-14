use crate::api;
use crate::auth::ProviderAuth;
use crate::i18n::Locale;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    #[default]
    DeepSeek,
    Kimi,
    BigModel,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::Kimi => "kimi",
            Self::BigModel => "bigmodel",
        }
    }

    pub fn capabilities(self) -> ProviderCapabilities {
        match self {
            Self::DeepSeek => ProviderCapabilities {
                platform_token: true,
                api_key: true,
                balance: true,
                models: true,
                monthly_usage: true,
                pricing: true,
            },
            Self::Kimi => ProviderCapabilities {
                platform_token: false,
                api_key: true,
                balance: true,
                models: true,
                monthly_usage: false,
                pricing: false,
            },
            Self::BigModel => ProviderCapabilities {
                platform_token: true,
                api_key: true,
                balance: false,
                models: false,
                monthly_usage: false,
                pricing: false,
            },
        }
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "deepseek" | "ds" => Ok(Self::DeepSeek),
            "kimi" | "moonshot" => Ok(Self::Kimi),
            "bigmodel" | "zhipu" | "zhipuai" => Ok(Self::BigModel),
            other => anyhow::bail!(
                "Unsupported provider: {other}. Expected one of: deepseek, kimi, bigmodel"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub platform_token: bool,
    pub api_key: bool,
    pub balance: bool,
    pub models: bool,
    pub monthly_usage: bool,
    pub pricing: bool,
}

pub fn unsupported(provider: ProviderId, capability: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "{} does not support {} yet. See README provider support matrix.",
        provider,
        capability
    )
}

pub fn provider_from_cli(value: &Option<String>) -> anyhow::Result<ProviderId> {
    value
        .as_deref()
        .unwrap_or(ProviderId::DeepSeek.as_str())
        .parse()
}

pub async fn summary(
    provider: ProviderId,
    auth: &ProviderAuth,
    locale: &Locale,
) -> anyhow::Result<(api::UserSummaryData, u64)> {
    match provider {
        ProviderId::DeepSeek => {
            let token = auth.platform_token.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Missing deepseek platform-token. Run: metrix auth <TOKEN>")
            })?;
            let now = chrono::Local::now();
            let (summary, amount) = tokio::try_join!(
                api::get_user_summary(token, locale),
                api::get_usage_amount(token, now.month(), now.year(), locale),
            )?;
            let total_requests = amount
                .total
                .iter()
                .flat_map(|m| &m.usage)
                .filter(|u| u.usage_type == api::USAGE_REQUEST)
                .filter_map(|u| u.amount.parse::<u64>().ok())
                .sum();
            Ok((summary, total_requests))
        }
        ProviderId::Kimi => {
            let api_key = auth
                .api_key
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Missing kimi api-key"))?;
            let balance = api::get_kimi_balance(api_key, locale).await?;
            Ok((api::summary_from_kimi_balance(balance), 0))
        }
        ProviderId::BigModel => Err(unsupported(provider, "summary")),
    }
}

pub async fn usage(
    provider: ProviderId,
    auth: &ProviderAuth,
    month: u32,
    year: i32,
    locale: &Locale,
) -> anyhow::Result<Vec<api::DaySummary>> {
    match provider {
        ProviderId::DeepSeek => {
            let token = auth.platform_token.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Missing deepseek platform-token. Run: metrix auth <TOKEN>")
            })?;
            let (amount, cost) = tokio::try_join!(
                api::get_usage_amount(token, month, year, locale),
                api::get_usage_cost(token, month, year, locale),
            )?;
            Ok(api::merge_usage(&amount, &cost))
        }
        ProviderId::Kimi | ProviderId::BigModel => Err(unsupported(provider, "monthly usage")),
    }
}

pub async fn models(
    provider: ProviderId,
    auth: &ProviderAuth,
    locale: &Locale,
) -> anyhow::Result<Vec<String>> {
    match provider {
        ProviderId::DeepSeek => {
            if let Some(api_key) = auth.api_key.as_deref() {
                api::get_models(api_key, locale).await
            } else {
                let token = auth.platform_token.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("Missing deepseek platform-token. Run: metrix auth <TOKEN>")
                })?;
                let now = chrono::Local::now();
                let amount = api::get_usage_amount(token, now.month(), now.year(), locale).await?;
                let days = api::merge_usage(&amount, &[]);
                let mut models: Vec<String> = days
                    .iter()
                    .map(|d| d.model.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                models.sort();
                Ok(models)
            }
        }
        ProviderId::Kimi => {
            let api_key = auth
                .api_key
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Missing kimi api-key"))?;
            api::get_kimi_models(api_key, locale).await
        }
        ProviderId::BigModel => Err(unsupported(provider, "models")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_provider_aliases() {
        assert_eq!(
            "deepseek".parse::<ProviderId>().unwrap(),
            ProviderId::DeepSeek
        );
        assert_eq!("moonshot".parse::<ProviderId>().unwrap(), ProviderId::Kimi);
        assert_eq!(
            "zhipuai".parse::<ProviderId>().unwrap(),
            ProviderId::BigModel
        );
    }

    #[test]
    fn exposes_capabilities() {
        let deepseek = ProviderId::DeepSeek.capabilities();
        assert!(deepseek.platform_token);
        assert!(deepseek.api_key);
        assert!(deepseek.monthly_usage);

        let kimi = ProviderId::Kimi.capabilities();
        assert!(!kimi.platform_token);
        assert!(kimi.balance);
        assert!(!kimi.monthly_usage);

        let bigmodel = ProviderId::BigModel.capabilities();
        assert!(bigmodel.platform_token);
        assert!(bigmodel.api_key);
        assert!(!bigmodel.balance);
    }
}
