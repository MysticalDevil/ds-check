# metrix

用于查询 AI 平台用量、余额和 API 消费的 CLI 工具。

## 功能特性

- 支持 DeepSeek、Kimi、BigModel provider adapter
- 在厂商公开或已确认接口支持时查看账户余额和月度消费
- 在支持的厂商上追踪 API 请求次数和 Token 用量
- 按模型和 Token 类型查看每日用量明细
- 模型筛选：列出所有模型、逐模型渲染表格、子串匹配
- **可选 API Key 支持**：在支持的厂商上获取全量模型列表
- **模型定价**：缓存 DeepSeek 每百万 tokens 价格表
  （运行 `python3 scripts/fetch_pricing.py` 更新）
- 多语言支持：zh_CN、zh_TW、en_US、ja_JP
- JSON 输出模式，便于脚本集成
- ASCII / Unicode 渲染模式
- 跨平台（Linux、macOS、Windows）

## 安装

```bash
cargo install --path .
```

## 用法

> 运行 `metrix --help` 查看带颜色的完整帮助信息。

### 厂商支持

| 厂商 | 凭据 | 概览 | 用量 | 模型 | 定价 | 说明 |
|---|---|---:|---:|---:|---:|---|
| `deepseek` | platform-token + api-key | 是 | 是 | 是 | 是 | 默认厂商。使用逆向的网页平台接口和 `api.deepseek.com/models`。 |
| `kimi` | api-key | 是 | 否 | 是 | 否 | 使用 Kimi 官方 API：`api.moonshot.cn/v1/users/me/balance` 和 `/models`。 |
| `bigmodel` | platform-token + api-key | 实验 | 实验 | 否 | 否 | 当前先存储凭据；财务中心接口形状捕获后再实现自动查询。 |

所有命令都可使用 `--provider deepseek|kimi|bigmodel`，默认是 `deepseek`。

### 查看用量概览

```bash
metrix summary
```

输出示例（Unicode 模式）：

```text
┏ DeepSeek 使用量 ━━━━━━━━━━━━━┓
┃                              ┃
┃   充值余额       121.76 CNY  ┃
┃   本月消费       10.30 CNY   ┃
┃   API 请求次数   2.58K       ┃
┃   Tokens         82.56M      ┃
┃                              ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

### 登录认证

```bash
# 保存平台 Token
metrix auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 显式保存 DeepSeek 平台 Token
metrix auth --provider deepseek --platform-token sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 交互式输入（会显示 Token 获取地址）
metrix auth

# 保存 DeepSeek API Key（需先完成 auth）
metrix apikey sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 保存 Kimi API Key
metrix auth --provider kimi --api-key sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 保存 BigModel 平台 Token 和 API Key
metrix auth --provider bigmodel --platform-token <web-token> --api-key <api-key>
```

Token 和用户信息存储在 `$XDG_CONFIG_HOME/metrix/auth.json`。

**两种凭证**：

- **平台 Token**：来自厂商网页控制台的 Bearer Token，用于厂商私有用量/财务接口。
- **API Key**：用于官方公开 API 或 OpenAI 兼容接口。

### 列出模型

```bash
# 从用量数据推导当月使用的模型，或配置了 API Key 时获取全量列表
metrix models

# Kimi 官方模型列表
metrix --provider kimi models
```

> 配置了 API Key（`metrix apikey <key>`）时，`models` 自动调用
> `api.deepseek.com/models` 获取完整列表。未配置时回退到用量推导模式，
> 并在 stderr 提示用户。

### 查看详细用量

```bash
# 当月用量（每个模型一张表）
metrix usage

# 指定月份
metrix usage -m 4 -y 2026

# 按模型筛选（子串匹配）
metrix usage -M v4-pro
metrix usage -M flash
```

`usage` 当前仅支持 DeepSeek。

### JSON 输出

```bash
metrix summary --json
metrix usage --json -m 5
metrix usage --json -M flash
metrix models --json
metrix price --json
```

### 切换语言

```bash
metrix --locale zh_CN summary
metrix --locale ja_JP usage
```

未指定时，自动从 `LANG` 环境变量检测语言。

支持的语言：`zh_CN`、`zh_TW`、`en_US`、`ja_JP`。

### 查看模型定价

```bash
# 显示定价表（需要缓存数据）
python3 scripts/fetch_pricing.py  # 先更新缓存
metrix price

# JSON 输出
metrix price --json
```

价格为每百万 tokens 的 CNY 单价。数据缓存于
`$XDG_CACHE_HOME/metrix/pricing.json`。运行
`python3 scripts/fetch_pricing.py` 获取。无需登录即可查看。

`price` 当前仅支持 DeepSeek。

### ASCII 渲染模式

```bash
# 纯 ASCII 表格，无 Unicode 边框和颜色
METRIX_RENDER=ascii metrix summary
METRIX_RENDER=ascii metrix usage
METRIX_RENDER=ascii metrix price
```

ASCII 模式强制使用英文标签，避免输出 Unicode 字符。

## 环境变量

| 变量 | 取值 | 说明 |
|---|---|---|
| `METRIX_MOCK` | `1` | 使用模拟数据，不发起网络请求 |
| `METRIX_RENDER` | `ascii` / `unicode` | 输出样式（默认：`unicode`） |
| `METRIX_LOCALE` | `zh_CN`、`zh_TW`、`en_US`、`ja_JP` | 设置默认语言 |
| `LANG` | 如 `zh_CN.UTF-8` | 未指定 `--locale` 时自动检测 |

## 厂商 API 来源

- DeepSeek 平台接口来自 `platform.deepseek.com` 的网页流量分析；
  API Key 模型列表使用 `https://api.deepseek.com/models`。
- Kimi 官方 API 概览：<https://platform.kimi.com/docs/api/overview>。
  余额：<https://platform.kimi.com/docs/api/balance>。
  模型列表：<https://platform.kimi.com/docs/api/list-models>。
- BigModel 官方 API 概览：<https://docs.bigmodel.cn/cn/api/introduction>。
  财务中心：<https://bigmodel.cn/finance-center/finance/overview>。
  财务中心需要 JavaScript/登录，当前尚未作为稳定 API adapter 实现。

## 开发

```bash
# 检查
cargo check

# 运行全部测试
cargo test

# 只运行单元测试
cargo test --lib

# 只运行集成测试
cargo test --test integration

# 构建发布版本
cargo build --release
```

## 许可证

MIT
