# ds-check — Agent Guide

> This file is for AI coding agents. Read this first before modifying the project.

## Project Overview

`ds-check` is a Rust CLI tool that queries the DeepSeek开放平台 (DeepSeek Platform) for account balance, monthly API usage, and token consumption details. It calls undocumented internal APIs discovered via traffic analysis of `platform.deepseek.com`.

- **Language**: Rust (Edition 2024)
- **Binary name**: `ds-check`
- **License**: MIT
- **Repository**: https://github.com/MysticalDevil/ds-check

## Technology Stack

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing with derive macros and colored help |
| `reqwest` | Async HTTP client for DeepSeek API calls |
| `tokio` | Async runtime (full features) |
| `serde` / `serde_json` | JSON serialization / deserialization |
| `anyhow` | Ergonomic error handling |
| `dirs` | XDG config directory resolution |
| `chrono` | Date/time handling |
| `ratatui` | Terminal UI widget construction (Table, Block, Row) |
| `crossterm` | Terminal ANSI color/style commands (queue, SetForegroundColor, etc.) |
| `sha2` | SHA-256 hashing for cache key derivation |
| `unicode-width` | CJK / wide-character display width calculation |

### Why crossterm is a direct dependency

ratatui renders widgets to an in-memory `Buffer`. The buffer stores logical `Cell` objects (char + `ratatui::style::Style`), not rendered ANSI escape codes. To produce colored terminal output, we iterate buffer cells at print time and use `crossterm::queue!` to emit `SetForegroundColor`, `SetAttribute(Bold)`, etc. for each cell's style. This is NOT something ratatui handles — the backend would, but we bypass the backend to avoid entering raw mode (we're a one-shot CLI, not an interactive TUI).

## Build & Run

```bash
# Check
cargo check

# Build debug
cargo build

# Build release (strip + LTO enabled in Cargo.toml)
cargo build --release

# Install locally
cargo install --path .

# Run with mock data (no network)
DSCHECK_MOCK=1 cargo run -- summary

# Run with ASCII output
DSCHECK_RENDER=ascii cargo run -- usage
```

## Project Structure

```
src/
├── main.rs   # CLI definition (clap), command dispatch, entry point
├── api.rs    # HTTP client, API structs, response merging logic, pricing loader
├── auth.rs   # Token persistence: load/save JSON to XDG config dir
├── cache.rs  # 60s TTL API response cache, SHA-256 keyed, XDG_CACHE_HOME
├── i18n.rs   # Hard-coded translations for 4 locales (zh_CN, zh_TW, en_US, ja_JP)
├── mock.rs   # Mock data generators for offline development / demo
└── output.rs # Rendering layer: Unicode cards (ratatui), ASCII plain text, JSON
```

### Module Responsibilities

- **`main.rs`** — Defines `Cli` and `Commands` via `clap` derive. Global flags: `--json`, `--locale`. Subcommands: `auth`, `summary`, `usage`, `models`, `price`. Reads env vars `DSCHECK_MOCK`, `DSCHECK_RENDER`, `DSCHECK_LOCALE`.
- **`api.rs`** — `BASE_URL = "https://platform.deepseek.com"`. Generic `api_get<T>()` helper. Endpoints:
  - `/auth-api/v0/users/current`
  - `/api/v0/users/get_user_summary`
  - `/api/v0/usage/amount?month={}&year={}`
  - `/api/v0/usage/cost?month={}&year={}`
  - `merge_usage()` combines amount + cost data into a flat `Vec<DaySummary>`.
  - `/models` on `api.deepseek.com` (via API Key, OpenAI-compatible)
  - `load_pricing()` reads cached `pricing.json` from `XDG_CACHE_HOME/ds-check/pricing.json`
- **`auth.rs`** — Stores `AuthConfig` (token, nickname, email, currency) as pretty-printed JSON at `$XDG_CONFIG_HOME/ds-check/auth.json`.
- **`i18n.rs`** — `Locale` enum with `from_str()`, `detect()` (reads `LANG`), and `t(key)` for localized strings.
- **`mock.rs`** — `mock_user_summary()`, `mock_usage_amount()`, `mock_usage_cost()`. Cost mock uses hard-coded rates: cache_hit 0.025/1M, cache_miss 0.55/1M, response 2.19/1M.
- **`output.rs`** — Three render paths:
  - **Unicode mode**: ratatui `Table`/`Block` widgets → in-memory `Buffer` → crossterm ANSI color commands for inline terminal output.
  - **ASCII mode**: Plain text with `=` headers and `|` delimited tables, no ANSI colors. Bypasses ratatui entirely because ratatui's `BorderType::Plain` uses Unicode single-line box-drawing characters (`┌│└` U+2500 series), not pure ASCII (`+-|`). There is no ratatui border set for pure ASCII.
  - **JSON mode**: Pretty-printed JSON via `serde_json`.

## Development Conventions

### Code Style

- Standard Rust naming: `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Use `anyhow::Result<T>` for fallible functions.
- Async functions use `tokio`; main is `#[tokio::main]`.
- Derive `Debug` and `Deserialize` (from `serde`) on API data structs.
- `#[allow(dead_code)]` is used on some API struct fields that are deserialized but not yet consumed in UI logic.

### Color Palette (output.rs)

When modifying UI output, respect the existing palette:

| Constant | Color | Usage |
|---|---|---|
| `C_BORDER` | Cyan | Table borders |
| `C_TITLE` | Yellow | Card titles |
| `C_LABEL` | Gray | Label text |
| `C_BALANCE` | Green | Balance values |
| `C_COST` | Yellow | Cost values |
| `C_REQUESTS` | Cyan | Request counts |
| `C_TOKENS` | White | Token counts |
| `C_HEADER_BG` | Rgb(50,50,55) | Table header background |
| `C_ROW_EVEN_BG` | Rgb(26,26,32) | Alternating row background |
| `C_COST_DIM` | Rgb(0xCC,0xAA,0x00) | Cost column in usage table |

ASCII mode strips all colors (`Color::Reset`). Note: ASCII mode currently overrides locale to `Locale::EnUS` — this is a known issue (REVIEW.md #6), CJK text is valid UTF-8 and should be preserved.

### Adding a New Locale

1. Add variant to `Locale` enum in `i18n.rs`.
2. Add branch in `from_str()`.
3. Add full message map in `messages()`.
4. Ensure all keys present in existing locales are included.

### Adding a New API Endpoint

1. Add response struct(s) in `api.rs` with `Deserialize`.
2. Add wrapper type if nested under `data.biz_data`.
3. Implement a thin `pub async fn` that calls `api_get::<T>(token, path)`.
4. Wire into `main.rs` command handler.

## Testing Strategy

**Unit tests** exist in `api.rs`, `auth.rs`, `i18n.rs`, `output.rs`. **Integration tests** in `tests/integration.rs` run the CLI binary with `DSCHECK_MOCK=1`.

Manual testing via mock mode:

```bash
# Mock auth
DSCHECK_MOCK=1 cargo run -- auth fake-token

# Mock summary
DSCHECK_MOCK=1 cargo run -- summary

# Mock usage
DSCHECK_MOCK=1 cargo run -- usage

# Mock usage with JSON
DSCHECK_MOCK=1 cargo run -- usage --json

# ASCII render test
DSCHECK_MOCK=1 DSCHECK_RENDER=ascii cargo run -- usage
```

If adding tests, prefer:
- `cargo test` with `#[tokio::test]` for async API client logic.
- Mock server (e.g., `wiremock`) or injected HTTP client for `api.rs`.
- Snapshot testing for `output.rs` render output.

## Security Considerations

- **Token storage**: The DeepSeek API token is stored in **plaintext** JSON at `$XDG_CONFIG_HOME/ds-check/auth.json`. Do not add encryption unless explicitly requested; document the risk instead.
- **Network**: All API calls use HTTPS. The `x-app-version` header is hard-coded to `"20240425.0"` to mimic the official web client.
- **Token exposure**: The token is passed as a CLI argument to `auth`. It may appear in shell history. An interactive prompt is available (`ds-check auth` with no args) to avoid this.
- **.env file**: A `.env` exists locally but is gitignored. It is **not** used by the application code; the program reads `DSCHECK_*` env vars directly at runtime.
- **API stability**: These are undocumented internal APIs. DeepSeek may change them without notice. The `API.md` file documents the observed schema as of 2026-05.

## Environment Variables

| Variable | Values | Description |
|---|---|---|
| `DSCHECK_MOCK` | `1` or `true` | Use mock data, skip all network requests |
| `DSCHECK_RENDER` | `ascii` / `unicode` | Output style (default: `unicode`) |
| `DSCHECK_LOCALE` | `zh_CN`, `zh_TW`, `en_US`, `ja_JP` | Override locale |
| `LANG` | e.g. `zh_CN.UTF-8` | Auto-detected locale when `--locale` is omitted |

## Deployment / Distribution

- `cargo install --path .` installs the binary to `~/.cargo/bin/`.
- Release profile enables `strip = true` and `lto = true` for smaller binaries.
- Cross-platform: Linux, macOS, Windows (no platform-specific code).

## Important Files

| File | Purpose |
|---|---|
| `Cargo.toml` | Package manifest, dependencies, release profile |
| `API.md` | Chinese-language documentation of reverse-engineered DeepSeek APIs |
| `README.md` / `README.zh_CN.md` | User-facing documentation |
| `.gitignore` | Ignores `/target`, `.env`, `REVIEW.md` |
