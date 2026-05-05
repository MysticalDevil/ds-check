mod api;
mod auth;
mod cache;
mod i18n;
mod mock;
mod output;

use anyhow::Context;
use chrono::Datelike;
use clap::{Parser, Subcommand};
use i18n::Locale;
use output::RenderMode;
use std::io::Write;

#[derive(Parser)]
#[command(
    name = "ds-check",
    version,
    about = "DeepSeek platform usage CLI tool",
    color = clap::ColorChoice::Auto,
    after_help = "Examples:\n  ds-check summary          Show usage summary\n  ds-check auth <TOKEN>     Authenticate with token\n  ds-check usage -m 5       Show May usage details\n  ds-check models           List all models used\n  ds-check price            Show model pricing\n  ds-check --json            Output as JSON\n\nEnv vars:\n  DSCHECK_MOCK=1            Use mock data (no network)\n  DSCHECK_RENDER=ascii|unicode  Output style (default: unicode)\n  DSCHECK_LOCALE=zh_CN      Set locale"
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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Save API token (validates and stores user info)")]
    Auth {
        #[arg(help = "DeepSeek API token (leave empty for interactive input)")]
        token: Option<String>,
        #[arg(long, help = "Optional DeepSeek API Key for api.deepseek.com endpoints")]
        api_key: Option<String>,
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
}

fn get_locale(cli: &Cli) -> Locale {
    if let Some(ref l) = cli.locale {
        return Locale::from_str(l);
    }
    if let Ok(l) = std::env::var("DSCHECK_LOCALE") {
        return Locale::from_str(&l);
    }
    Locale::detect()
}

fn is_mock() -> bool {
    std::env::var("DSCHECK_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let locale = get_locale(&cli);
    let render_mode = RenderMode::from_env();

    // ASCII mode forces English locale because manual println tables
    // use fixed-width alignment that breaks with CJK characters
    let locale = if render_mode == output::RenderMode::Ascii {
        Locale::EnUS
    } else {
        locale
    };

    match &cli.command {
        Some(Commands::Auth { token, api_key }) => {
            cmd_auth(token, api_key, &locale).await?;
        }
        Some(Commands::Summary) => {
            cmd_summary(cli.json, &locale, render_mode).await?;
        }
        Some(Commands::Usage { month, year, model }) => {
            cmd_usage(
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
            cmd_models(cli.json, &locale).await?;
        }
        Some(Commands::Price) => {
            cmd_price(cli.json, &locale, render_mode)?;
        }
        None => {
            println!("{}", locale.t("use_help"));
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_price(json: bool, locale: &Locale, render_mode: RenderMode) -> anyhow::Result<()> {
    let data = api::load_pricing()
        .with_context(|| locale.t("pricing_not_found"))?;
    output::print_pricing(&data, json, *locale, render_mode)?;
    Ok(())
}

async fn cmd_auth(token_opt: &Option<String>, api_key_opt: &Option<String>, locale: &Locale) -> anyhow::Result<()> {
    let token = match token_opt {
        Some(t) => t.clone(),
        None => {
            println!("{}", locale.t("token_help"));
            print!("{}", locale.t("enter_token"));
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if token.is_empty() {
        anyhow::bail!(locale.t("invalid_token"));
    }

    let (nickname, email, currency) = if is_mock() {
        (
            "MockUser".to_string(),
            "mock@example.com".to_string(),
            "CNY".to_string(),
        )
    } else {
        let user = api::get_current_user(&token, locale)
            .await
            .context(locale.t("invalid_token"))?;
        (user.id_profile.name, user.email, user.currency)
    };

    let config = auth::AuthConfig {
        token,
        nickname: nickname.clone(),
        email,
        currency,
        api_key: api_key_opt.clone(),
    };

    auth::save(&config)?;
    println!("{}", locale.t("auth_success").replace("{}", &nickname));
    println!(
        "{}",
        locale
            .t("token_saved")
            .replace("{}", &auth::config_path_str())
    );
    if api_key_opt.is_some() {
        println!("{}", locale.t("api_key_saved"));
    }

    Ok(())
}

async fn cmd_summary(json: bool, locale: &Locale, render_mode: RenderMode) -> anyhow::Result<()> {
    let config = auth::load()?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;

    let (summary, amount) = if is_mock() {
        (mock::mock_user_summary(), mock::mock_usage_amount())
    } else {
        let now = chrono::Local::now();
        tokio::try_join!(
            api::get_user_summary(&config.token, locale),
            api::get_usage_amount(&config.token, now.month(), now.year(), locale),
        )?
    };

    let total_requests: u64 = amount
        .total
        .iter()
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == api::USAGE_REQUEST)
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum();

    output::print_summary(&summary, total_requests, json, *locale, render_mode)?;
    Ok(())
}

async fn cmd_usage(
    month: Option<u32>,
    year: Option<i32>,
    model: Option<&str>,
    json: bool,
    locale: &Locale,
    render_mode: RenderMode,
) -> anyhow::Result<()> {
    let config = auth::load()?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;

    let now = chrono::Local::now();
    let month = month.unwrap_or(now.month());
    let year = year.unwrap_or(now.year());

    let (amount, cost) = if is_mock() {
        (mock::mock_usage_amount(), mock::mock_usage_cost())
    } else {
        tokio::try_join!(
            api::get_usage_amount(&config.token, month, year, locale),
            api::get_usage_cost(&config.token, month, year, locale),
        )?
    };

    let days = api::merge_usage(&amount, &cost);

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

async fn cmd_models(json: bool, locale: &Locale) -> anyhow::Result<()> {
    let config = auth::load()?
        .ok_or_else(|| anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint")))?;

    // Prefer API Key route for full model list
    let models: Vec<String> = if let Some(ref api_key) = config.api_key {
        if is_mock() {
            mock::mock_api_models()
        } else {
            api::get_models(api_key).await?
        }
    } else {
        // Fallback: derive models from current month usage data
        let now = chrono::Local::now();

        let amount = if is_mock() {
            mock::mock_usage_amount()
        } else {
            api::get_usage_amount(&config.token, now.month(), now.year(), locale).await?
        };

        let days = api::merge_usage(&amount, &[]);
        let mut m: Vec<String> = days
            .iter()
            .map(|d| d.model.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        m.sort();
        m
    };

    if json {
        let output: Vec<serde_json::Value> = models
            .iter()
            .map(|m| serde_json::json!({"model": m}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for m in models {
            println!("{}", m);
        }
    }

    if config.api_key.is_none() {
        eprintln!("* {}", locale.t("api_key_hint"));
    }

    Ok(())
}
