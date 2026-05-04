mod api;
mod auth;
mod i18n;
mod output;

use anyhow::Context;
use chrono::Datelike;
use clap::{Parser, Subcommand};
use i18n::Locale;
use std::io::Write;

#[derive(Parser)]
#[command(
    name = "ds-check",
    version,
    about = "DeepSeek platform usage CLI tool",
    after_help = "Examples:\n  ds-check                  Show usage summary\n  ds-check auth <TOKEN>     Authenticate with token\n  ds-check usage -m 5       Show May usage details\n  ds-check --json           Output as JSON"
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
    },
    #[command(about = "Show detailed usage by day and model")]
    Usage {
        #[arg(short, long, help = "Month (1-12, default: current)")]
        month: Option<u32>,
        #[arg(short, long, help = "Year (default: current)")]
        year: Option<i32>,
        #[arg(short = 'M', long, help = "Filter by model name")]
        model: Option<String>,
    },
}

fn get_locale(cli: &Cli) -> Locale {
    if let Some(ref l) = cli.locale {
        Locale::from_str(l)
    } else {
        Locale::detect()
    }
}

fn auth_config_path() -> String {
    auth::config_path_str()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let locale = get_locale(&cli);

    match &cli.command {
        Some(Commands::Auth { token }) => {
            cmd_auth(token, &locale).await?;
        }
        Some(Commands::Usage {
            month,
            year,
            model,
        }) => {
            cmd_usage(*month, *year, model.as_deref(), cli.json, &locale).await?;
        }
        None => {
            cmd_summary(cli.json, &locale).await?;
        }
    }

    Ok(())
}

async fn cmd_auth(token_opt: &Option<String>, locale: &Locale) -> anyhow::Result<()> {
    let token = match token_opt {
        Some(t) => t.clone(),
        None => {
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

    let user = api::get_current_user(&token)
        .await
        .context(locale.t("invalid_token"))?;

    let config = auth::AuthConfig {
        token: token.clone(),
        nickname: user.id_profile.name,
        email: user.email,
        currency: user.currency,
    };

    auth::save(&config)?;
    println!(
        "{}",
        locale.t("auth_success").replace("{}", &config.nickname)
    );
    println!("{}", locale.t("token_saved").replace("{}", &auth_config_path()));

    Ok(())
}

async fn cmd_summary(json: bool, locale: &Locale) -> anyhow::Result<()> {
    let config = auth::load()
        .ok_or_else(|| {
            anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint"))
        })?;

    let now = chrono::Local::now();
    let month = now.month();
    let year = now.year();

    let (summary, amount) = tokio::try_join!(
        api::get_user_summary(&config.token),
        api::get_usage_amount(&config.token, month, year),
    )?;

    let total_requests: u64 = amount
        .total
        .iter()
        .flat_map(|m| &m.usage)
        .filter(|u| u.usage_type == "REQUEST")
        .filter_map(|u| u.amount.parse::<u64>().ok())
        .sum();

    output::print_summary(&summary, total_requests, &config.nickname, json, *locale);

    Ok(())
}

async fn cmd_usage(
    month: Option<u32>,
    year: Option<i32>,
    model: Option<&str>,
    json: bool,
    locale: &Locale,
) -> anyhow::Result<()> {
    let config = auth::load()
        .ok_or_else(|| {
            anyhow::anyhow!("{}\n{}", locale.t("no_token"), locale.t("auth_hint"))
        })?;

    let now = chrono::Local::now();
    let month = month.unwrap_or(now.month());
    let year = year.unwrap_or(now.year());

    let (amount, cost) = tokio::try_join!(
        api::get_usage_amount(&config.token, month, year),
        api::get_usage_cost(&config.token, month, year),
    )?;

    let days = api::merge_usage(&amount, &cost);
    output::print_usage(&days, model, json, *locale);

    Ok(())
}
