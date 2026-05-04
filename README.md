# ds-check

A CLI tool for checking DeepSeek platform usage, balance, and API consumption.

## Features

- View account balance and monthly costs
- Track API request counts and token usage
- Detailed daily usage breakdown by model and token type
- Multi-locale support: zh_CN, zh_TW, en_US, ja_JP
- JSON output mode for scripting
- Cross-platform (Linux, macOS, Windows)

## Installation

```bash
cargo install --path .
```

## Usage

### Check balance and usage

```bash
ds-check
```

Output:

```
============ DeepSeek Usage ============
       User: nickname
    Balance: 121.75 CNY
Monthly Cost: 10.30 CNY
API Requests: 785
     Tokens: 82.56M
========================================
```

### Authenticate

```bash
# Provide token directly
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Interactive prompt
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
ds-check --json
ds-check usage --json -m 5
```

### Set locale

```bash
ds-check --locale zh_CN
ds-check --locale ja_JP
```

Locale auto-detects from `LANG` environment variable when not specified.

Supported locales: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`.

## License

MIT
