# ds-check

A CLI tool for checking DeepSeek platform usage, balance, and API consumption.

[中文文档](README.zh_CN.md)

## Features

- View account balance and monthly costs (colorful card UI)
- Track API request counts and token usage
- Detailed daily usage breakdown by model and token type
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
┃   Balance       119.00 CNY  ┃
┃   Monthly Cost  13.06 CNY   ┃
┃   API Requests  950         ┃
┃   Tokens        106.73M     ┃
┃                             ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

### Authenticate

```bash
# Provide token directly
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Interactive prompt (shows token URL)
ds-check auth
```

Token and user info are stored at `$XDG_CONFIG_HOME/ds-check/auth.json`.

### View detailed usage

```bash
# Current month
ds-check usage

# Specific month
ds-check usage -m 4 -y 2026

# Filter by model
ds-check usage -M v4-pro
```

### Output as JSON

```bash
ds-check summary --json
ds-check usage --json -m 5
```

### Set locale

```bash
ds-check --locale zh_CN summary
ds-check --locale ja_JP usage
```

Locale auto-detects from `LANG` environment variable when not specified.

Supported locales: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`.

## Environment Variables

| Variable | Description |
|---|---|
| `DSCHECK_MOCK` | Set to `1` to use mock data (no network calls) |
| `DSCHECK_RENDER` | Output style: `ascii` or `unicode` (default: `unicode`) |
| `DSCHECK_LOCALE` | Set default locale, e.g. `zh_CN`, `en_US` |

## License

MIT
