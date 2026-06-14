# metrix

A CLI tool for checking AI provider usage, balance, and API consumption.

[中文文档](README.zh_CN.md)

## Features

- Provider adapters for DeepSeek, Kimi, and BigModel
- View account balance and monthly costs where the provider exposes them
- Track API request counts and token usage for supported providers
- Detailed daily usage breakdown by model and token type
- Model filtering: list all models, render per-model tables, or substring match
- **Optional API Key support**: get full model lists where supported
- **Model pricing**: cached DeepSeek pricing table per 1M tokens
  (run `python3 scripts/fetch_pricing.py` to update)
- Multi-locale support: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`
- JSON output mode for scripting
- ASCII / Unicode render modes
- Cross-platform (Linux, macOS, Windows)

## Installation

```bash
cargo install --path .
```

## Usage

> Run `metrix --help` for colored CLI help with all options.

### Provider support

| Provider | Credentials | Summary | Usage | Models | Pricing | Notes |
|---|---|---:|---:|---:|---:|---|
| `deepseek` | platform-token + api-key | Yes | Yes | Yes | Yes | Default provider. Uses reverse-engineered platform APIs plus `api.deepseek.com/models`. |
| `kimi` | api-key | Yes | No | Yes | No | Uses official Kimi APIs: `api.moonshot.cn/v1/users/me/balance` and `/models`. |
| `bigmodel` | platform-token + api-key | Experimental | Experimental | No | No | Credentials are stored now; finance-center APIs are not implemented until request shapes are captured. |

Use `--provider deepseek|kimi|bigmodel` on any command. The default is `deepseek`.

### View usage summary

```bash
metrix summary
```

Output (Unicode mode):

```text
┏ DeepSeek Usage ━━━━━━━━━━━━━┓
┃                             ┃
┃   Balance       121.76 CNY  ┃
┃   Monthly Cost  10.30 CNY   ┃
┃   API Requests  2.58K       ┃
┃   Tokens        82.56M      ┃
┃                             ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

### Authenticate

```bash
# Save platform token
metrix auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Save an explicit DeepSeek platform token
metrix auth --provider deepseek --platform-token sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Interactive prompt (shows token URL)
metrix auth

# Save DeepSeek API Key for full model list
metrix apikey sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Save Kimi API Key
metrix auth --provider kimi --api-key sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Save BigModel platform token and API Key
metrix auth --provider bigmodel --platform-token <web-token> --api-key <api-key>
```

Token and user info are stored at `$XDG_CONFIG_HOME/metrix/auth.json`.

Credential types:

- **Platform token**: Bearer token from a provider web console, used for
  provider-private usage/billing APIs.
- **API Key**: public API key used for official OpenAI-compatible endpoints.

### List models

```bash
# Models used in current month (derived from usage data, or full list via API Key)
metrix models

# Kimi official model list
metrix --provider kimi models
```

> When an API Key is configured (`metrix apikey <key>` after auth), `models`
> automatically calls `api.deepseek.com/models` for the complete list. Without
> an API Key, it falls back to deriving models from usage data and shows a hint
> on stderr.

### View detailed usage

```bash
# Current month (one table per model)
metrix usage

# Specific month
metrix usage -m 4 -y 2026

# Filter by model (substring match)
metrix usage -M v4-pro
metrix usage -M flash
```

`usage` is currently supported by DeepSeek only.

### Output as JSON

```bash
metrix summary --json
metrix usage --json -m 5
metrix usage --json -M flash
metrix models --json
metrix price --json
```

### Set locale

```bash
metrix --locale zh_CN summary
metrix --locale ja_JP usage
```

Locale auto-detects from `LANG` environment variable when not specified.

Supported locales: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`.

### View model pricing

```bash
# Show pricing table (requires cached data)
python3 scripts/fetch_pricing.py  # update cache first
metrix price

# JSON output
metrix price --json
```

Prices are per 1M tokens in CNY. Data is cached at
`$XDG_CACHE_HOME/metrix/pricing.json`. Run
`python3 scripts/fetch_pricing.py` to populate it. No login required.

`price` is currently supported by DeepSeek only.

### ASCII render mode

```bash
# Plain ASCII tables, no Unicode borders or colors
METRIX_RENDER=ascii metrix summary
METRIX_RENDER=ascii metrix usage
METRIX_RENDER=ascii metrix price
```

ASCII mode forces English labels for pure ASCII output.

## Environment Variables

| Variable | Values | Description |
|---|---|---|
| `METRIX_MOCK` | `1` | Use mock data (no network calls) |
| `METRIX_RENDER` | `ascii` / `unicode` | Output style (default: `unicode`) |
| `METRIX_LOCALE` | `zh_CN`, `zh_TW`, `en_US`, `ja_JP` | Set default locale |
| `LANG` | e.g. `zh_CN.UTF-8` | Auto-detected locale when `--locale` is omitted |

## Provider API sources

- DeepSeek platform APIs are reverse-engineered from `platform.deepseek.com`;
  API Key model listing uses `https://api.deepseek.com/models`.
- Kimi official API overview: <https://platform.kimi.com/docs/api/overview>.
  Balance: <https://platform.kimi.com/docs/api/balance>.
  Models: <https://platform.kimi.com/docs/api/list-models>.
- BigModel official API overview:
  <https://docs.bigmodel.cn/cn/api/introduction>. Finance center:
  <https://bigmodel.cn/finance-center/finance/overview>. The finance center
  requires JavaScript/login and is not implemented as a stable API adapter yet.

## Development

```bash
# Check
cargo check

# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Build release
cargo build --release
```

## License

MIT
