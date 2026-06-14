# Changelog

All notable changes to metrix.

## Unreleased

### Added

- Provider adapter layer with `--provider deepseek|kimi|bigmodel`.
- Kimi API Key support for balance and model list via official Kimi APIs.
- BigModel platform-token/API-key storage with explicit experimental support notes.
- Versioned multi-provider auth config with backward-compatible DeepSeek
  config loading.

## [0.1.0] - 2026-05-21

### Added

- **6 subcommands**: `auth`, `apikey`, `summary`, `usage`, `models`, `price`
- Colored Unicode table output via ratatui + crossterm
- ASCII render mode (`METRIX_RENDER=ascii`)
- JSON output mode (`--json`) for all commands
- 4-locale i18n: zh_CN, zh_TW, en_US, ja_JP (auto-detect from `LANG`)
- API Key support via `metrix apikey` for full model list from `api.deepseek.com/models`
- Model pricing table (`metrix price`) with cached `pricing.json`
- `scripts/fetch_pricing.py` to scrape DeepSeek pricing from official docs
- HTTP API response caching (60s TTL, SHA-256 keyed, `XDG_CACHE_HOME`)
- Mock mode (`METRIX_MOCK=1`) for offline development and demos
- Interactive token prompt (`metrix auth` with no args)
- Model substring filtering in `usage` command

### Security

- Zero `unsafe` in production code
- Zero `unwrap()` / `expect()` in production code
- Plaintext token storage at `XDG_CONFIG_HOME/metrix/auth.json` (documented risk)
- All API calls over HTTPS

### Tests

- 41 unit tests covering: API client, HTTP mock (wiremock), cache, auth,
  i18n, output formatting, merge logic, mock data correctness
- 19 integration tests covering all subcommands, JSON output, ASCII render,
  locale override, model filtering

[0.1.0]: https://github.com/MysticalDevil/metrix/releases/tag/v0.1.0
