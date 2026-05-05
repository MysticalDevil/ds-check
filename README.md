# ds-check

A CLI tool for checking DeepSeek platform usage, balance, and API consumption.

[中文文档](README.zh_CN.md)

## Features

- View account balance and monthly costs (colorful card UI)
- Track API request counts and token usage
- Detailed daily usage breakdown by model and token type
- Model filtering: list all models, render per-model tables, or substring match
- **Optional API Key support**: get full model list via `api.deepseek.com/models`
- **Model pricing**: cached pricing table per 1M tokens (run `python3 scripts/fetch_pricing.py` to update)
- Multi-locale support: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`
- JSON output mode for scripting
- ASCII / Unicode render modes
- Cross-platform (Linux, macOS, Windows)

## Installation

```bash
cargo install --path .
```

## Usage

> Run `ds-check --help` for colored CLI help with all options.

### View usage summary

```bash
ds-check summary
```

Output (Unicode mode):

```
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
# Provide platform token directly
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Also save an API Key for api.deepseek.com endpoints
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx --api-key sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Interactive prompt (shows token URL)
ds-check auth
```

Token and user info are stored at `$XDG_CONFIG_HOME/ds-check/auth.json`.

**Two credential types**:
- **Platform token** (`<token>`): Bearer token from `platform.deepseek.com`, used for usage/balance data.
- **API Key** (`--api-key`): Key from `platform.deepseek.com/api_keys`, used for full model list via OpenAI-compatible API.

### List models

```bash
# Models used in current month (derived from usage data, or full list via API Key)
ds-check models
```

> When an API Key is configured (`ds-check auth <token> --api-key <key>`), `models` automatically calls `api.deepseek.com/models` for the complete list. Without an API Key, it falls back to deriving models from usage data and shows a hint on stderr.

### View detailed usage

```bash
# Current month (one table per model)
ds-check usage

# Specific month
ds-check usage -m 4 -y 2026

# Filter by model (substring match)
ds-check usage -M v4-pro
ds-check usage -M flash
```

### Output as JSON

```bash
ds-check summary --json
ds-check usage --json -m 5
ds-check usage --json -M flash
ds-check models --json
ds-check price --json
```

### Set locale

```bash
ds-check --locale zh_CN summary
ds-check --locale ja_JP usage
```

Locale auto-detects from `LANG` environment variable when not specified.

Supported locales: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`.

### View model pricing

```bash
# Show pricing table (requires cached data)
python3 scripts/fetch_pricing.py  # update cache first
ds-check price

# JSON output
ds-check price --json
```

Prices are per 1M tokens in CNY. Data is cached at `$XDG_CACHE_HOME/ds-check/pricing.json` (run `python3 scripts/fetch_pricing.py` to populate). No login required.

### ASCII render mode

```bash
# Plain ASCII tables, no Unicode borders or colors
DSCHECK_RENDER=ascii ds-check summary
DSCHECK_RENDER=ascii ds-check usage
DSCHECK_RENDER=ascii ds-check price
```

ASCII mode forces English labels for pure ASCII output.

## Environment Variables

| Variable | Values | Description |
|---|---|---|
| `DSCHECK_MOCK` | `1` | Use mock data (no network calls) |
| `DSCHECK_RENDER` | `ascii` / `unicode` | Output style (default: `unicode`) |
| `DSCHECK_LOCALE` | `zh_CN`, `zh_TW`, `en_US`, `ja_JP` | Set default locale |
| `LANG` | e.g. `zh_CN.UTF-8` | Auto-detected locale when `--locale` is omitted |

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
