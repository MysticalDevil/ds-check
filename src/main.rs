mod api;
mod auth;
mod cache;
mod i18n;
mod mock;
mod output;
mod provider;

use anyhow::Context;
use chrono::Datelike;
use clap::{Parser, Subcommand};
use i18n::Locale;
use output::RenderMode;
use provider::ProviderId;
use std::io::Write;

#[derive(Parser)]
#[command(
    name = "metrix",
    version,
    about = "AI provider usage CLI tool",
    color = clap::ColorChoice::Auto,
    disable_help_subcommand = true,
    after_help = "Examples:\n  metrix summary                    Show usage summary\n  metrix auth <TOKEN>               Save DeepSeek platform token\n  metrix auth --provider kimi --api-key <KEY>\n  metrix --provider kimi models     List Kimi models\n  metrix usage -m 5                 Show May usage details\n  metrix price                      Show model pricing\n  metrix --json                     Output as JSON\n\nEnv vars:\n  METRIX_MOCK=1                     Use mock data (no network)\n  METRIX_RENDER=ascii|unicode       Output style (default: unicode)\n  METRIX_LOCALE=zh_CN               Set locale"
)]
struct Cli {
    #[arg(short, long, global = true, help = "Output as JSON")]
    json: bool,

    #[arg(
        long,
        global = true,
        help = "Set output locale (zh_CN, zh_TW, en_US, ja_JP)"
    )]
    locale: Option<String>,

    #[arg(
        long,
        global = true,
        default_value = "deepseek",
        help = "Set provider (deepseek, kimi, bigmodel)"
    )]
    provider: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Save platform token (validates and stores user info)")]
    Auth {
        #[arg(help = "Provider token shorthand")]
        token: Option<String>,
        #[arg(long, help = "Provider web/platform Bearer token")]
        platform_token: Option<String>,
        #[arg(long, help = "Provider public API Key")]
        api_key: Option<String>,
    },
    #[command(about = "Save API Key for api.deepseek.com endpoints")]
    Apikey {
        #[arg(help = "DeepSeek API Key")]
        key: String,
    },
    #[command(about = "Show usage summary (balance, monthly cost, requests)")]
    Summary,
    #[command(about = "Show detailed usage by day and model")]
    Usage {
        #[arg(short, long, help = "Month (1-12, default: current)")]
        month: Option<u32>,
        #[arg(short, long, help = "Year (default: current)")]
        year: Option<i32>,
        #[arg(short = 'M', long, help = "Filter by model name")]
        model: Option<String>,
    },
    #[command(about = "List all models used in the current month")]
    Models,
    #[command(about = "Show model pricing per 1M tokens")]
    Price,
    #[command(about = "Show help")]
    Help,
}

fn get_locale(cli: &Cli) -> Locale {
    if let Some(ref l) = cli.locale {
        return Locale::from_str(l);
    }
    if let Ok(l) = std::env::var("METRIX_LOCALE") {
        return Locale::from_str(&l);
    }
    Locale::detect()
}

fn is_mock() -> bool {
    std::env::var("METRIX_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let locale = get_locale(&cli);
    let render_mode = RenderMode::from_env();
    let provider = provider::provider_from_cli(&Some(cli.provider.clone()))?;

    // ASCII mode forces English locale because manual println tables
    // use fixed-width alignment that breaks with CJK characters
    let locale = if render_mode == output::RenderMode::Ascii {
        Locale::EnUS
    } else {
        locale
    };

    match &cli.command {
        Some(Commands::Auth {
            token,
            platform_token,
            api_key,
        }) => {
            cmd_auth(provider, token, platform_token, api_key, &locale).await?;
        }
        Some(Commands::Apikey { key }) => {
            cmd_apikey(key, &locale)?;
        }
        Some(Commands::Summary) => {
            cmd_summary(provider, cli.json, &locale, render_mode).await?;
        }
        Some(Commands::Usage { month, year, model }) => {
            cmd_usage(
                provider,
                *month,
                *year,
                model.as_deref(),
                cli.json,
                &locale,
                render_mode,
            )
            .await?;
        }
        Some(Commands::Models) => {
            cmd_models(provider, cli.json, &locale).await?;
        }
        Some(Commands::Price) => {
            cmd_price(provider, cli.json, &locale, render_mode)?;
        }
        Some(Commands::Help) | None => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}

fn cmd_price(
    provider: ProviderId,
    json: bool,
    locale: &Locale,
    render_mode: RenderMode,
) -> anyhow::Result<()> {
    if provider != ProviderId::DeepSeek {
        return Err(provider::unsupported(provider, "pricing"));
    }
    let data = api::load_pricing().with_context(|| locale.t("pricing_not_found"))?;
    output::print_pricing(&data, json, *locale, render_mode)?;
    Ok(())
}

async fn cmd_auth(
    provider: ProviderId,
    token_opt: &Option<String>,
    platform_token_opt: &Option<String>,
    api_key_opt: &Option<String>,
    locale: &Locale,
) -> anyhow::Result<()> {
    let caps = provider.capabilities();
    let positional = token_opt.clone();
    let platform_token = platform_token_opt
        .clone()
        .or_else(|| positional.clone().filter(|_| caps.platform_token));
    let api_key = api_key_opt
        .clone()
        .or_else(|| positional.clone().filter(|_| !caps.platform_token));

    if platform_token.is_none() && api_key.is_none() && provider != ProviderId::DeepSeek {
        anyhow::bail!(
            "Missing credential. Use --api-key for kimi, or --platform-token/--api-key for bigmodel."
        );
    }

    let platform_token = if platform_token.is_none() && api_key.is_none() {
        match token_opt {
            Some(t) => Some(t.clone()),
            None => {
                println!("{}", locale.t("token_help"));
                print!("{}", locale.t("enter_token"));
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                Some(input.trim().to_string())
            }
        }
    } else {
        platform_token
    };

    let mut existing = auth::load_provider(provider)?.unwrap_or_default();

    if let Some(token) = platform_token {
        if token.is_empty() {
            anyhow::bail!(locale.t("invalid_token"));
        }
        if provider == ProviderId::DeepSeek && !is_mock() {
            let user = api::get_current_user(&token, locale)
                .await
                .context(locale.t("invalid_token"))?;
            existing.nickname = Some(user.id_profile.name);
            existing.email = Some(user.email);
            existing.currency = Some(user.currency);
        } else if provider == ProviderId::DeepSeek {
            existing.nickname = Some("MockUser".to_string());
            existing.email = Some("mock@example.com".to_string());
            existing.currency = Some("CNY".to_string());
        }
        existing.platform_token = Some(token);
    }

    if let Some(key) = api_key {
        if key.is_empty() {
            anyhow::bail!(locale.t("invalid_token"));
        }
        existing.api_key = Some(key);
    }

    auth::save_provider(provider, existing.clone())?;

    let name = existing.nickname.unwrap_or_else(|| provider.to_string());
    println!("{}", locale.t("auth_success").replace("{}", &name));
    println!(
        "{}",
        locale
            .t("token_saved")
            .replace("{}", &auth::config_path_str())
    );

    Ok(())
}

fn cmd_apikey(api_key: &str, locale: &Locale) -> anyhow::Result<()> {
    let mut auth = auth::load_provider(ProviderId::DeepSeek)?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;
    auth.api_key = Some(api_key.to_string());
    auth::save_provider(ProviderId::DeepSeek, auth)?;
    println!("{}", locale.t("api_key_saved"));
    Ok(())
}

async fn cmd_summary(
    provider: ProviderId,
    json: bool,
    locale: &Locale,
    render_mode: RenderMode,
) -> anyhow::Result<()> {
    let auth = auth::load_provider(provider)?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;

    let (summary, total_requests) = if is_mock() {
        match provider {
            ProviderId::DeepSeek => {
                let amount = mock::mock_usage_amount();
                let total_requests: u64 = amount
                    .total
                    .iter()
                    .flat_map(|m| &m.usage)
                    .filter(|u| u.usage_type == api::USAGE_REQUEST)
                    .filter_map(|u| u.amount.parse::<u64>().ok())
                    .sum();
                (mock::mock_user_summary(), total_requests)
            }
            ProviderId::Kimi => (mock::mock_kimi_summary(), 0),
            ProviderId::BigModel => return Err(provider::unsupported(provider, "summary")),
        }
    } else {
        provider::summary(provider, &auth, locale).await?
    };

    output::print_summary(&summary, total_requests, json, *locale, render_mode)?;
    Ok(())
}

async fn cmd_usage(
    provider: ProviderId,
    month: Option<u32>,
    year: Option<i32>,
    model: Option<&str>,
    json: bool,
    locale: &Locale,
    render_mode: RenderMode,
) -> anyhow::Result<()> {
    let auth = auth::load_provider(provider)?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;

    let now = chrono::Local::now();
    let month = month.unwrap_or(now.month());
    let year = year.unwrap_or(now.year());

    let days = if is_mock() {
        if provider != ProviderId::DeepSeek {
            return Err(provider::unsupported(provider, "monthly usage"));
        }
        api::merge_usage(&mock::mock_usage_amount(), &mock::mock_usage_cost())
    } else {
        provider::usage(provider, &auth, month, year, locale).await?
    };

    if let Some(filter) = model {
        output::print_usage(&days, Some(filter), json, *locale, render_mode)?;
    } else {
        let mut models: Vec<&str> = days
            .iter()
            .map(|d| d.model.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        models.sort();
        if models.is_empty() {
            println!("{}", locale.t("no_data"));
            return Ok(());
        }
        for m in models {
            output::print_usage(&days, Some(m), json, *locale, render_mode)?;
        }
    }

    Ok(())
}

async fn cmd_models(provider: ProviderId, json: bool, locale: &Locale) -> anyhow::Result<()> {
    let auth = auth::load_provider(provider)?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;

    let models: Vec<String> = if is_mock() {
        match provider {
            ProviderId::DeepSeek => mock::mock_api_models(),
            ProviderId::Kimi => mock::mock_kimi_models(),
            ProviderId::BigModel => return Err(provider::unsupported(provider, "models")),
        }
    } else {
        provider::models(provider, &auth, locale).await?
    };

    output::print_models(&models, json, *locale)?;

    if provider == ProviderId::DeepSeek && auth.api_key.is_none() {
        eprintln!("* {}", locale.t("api_key_hint"));
    }

    Ok(())
}
