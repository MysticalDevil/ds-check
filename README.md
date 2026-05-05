# ds-check

A CLI tool for checking DeepSeek platform usage, balance, and API consumption.

[中文文档](README.zh_CN.md)

## Features

- View account balance and monthly costs (colorful card UI)
- Track API request counts and token usage
- Detailed daily usage breakdown by model and token type
- Model filtering: list all models, render per-model tables, or substring match
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
# Provide token directly
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Interactive prompt (shows token URL)
ds-check auth
```

Token and user info are stored at `$XDG_CONFIG_HOME/ds-check/auth.json`.

### List models

```bash
# Models used in current month
ds-check models

# Models used in a specific month
ds-check models -m 4 -y 2026
```

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
```

### Set locale

```bash
ds-check --locale zh_CN summary
ds-check --locale ja_JP usage
```

Locale auto-detects from `LANG` environment variable when not specified.

Supported locales: `zh_CN`, `zh_TW`, `en_US`, `ja_JP`.

### ASCII render mode

```bash
# Plain ASCII tables, no Unicode borders or colors
DSCHECK_RENDER=ascii ds-check summary
DSCHECK_RENDER=ascii ds-check usage
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
