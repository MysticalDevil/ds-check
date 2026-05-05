# ds-check

用于查询 DeepSeek 开放平台用量、余额和 API 消费的 CLI 工具。

## 功能特性

- 查看账户余额和月度消费（彩色卡片 UI）
- 追踪 API 请求次数和 Token 用量
- 按模型和 Token 类型查看每日用量明细
- 模型筛选：列出所有模型、逐模型渲染表格、子串匹配
- 多语言支持：zh_CN、zh_TW、en_US、ja_JP
- JSON 输出模式，便于脚本集成
- ASCII / Unicode 渲染模式
- 跨平台（Linux、macOS、Windows）

## 安装

```bash
cargo install --path .
```

## 用法

> 运行 `ds-check --help` 查看带颜色的完整帮助信息。

### 查看用量概览

```bash
ds-check summary
```

输出示例（Unicode 模式）：

```
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
# 直接传入 Token
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 交互式输入（会显示 Token 获取地址）
ds-check auth
```

Token 和用户信息存储在 `$XDG_CONFIG_HOME/ds-check/auth.json`。

### 查看详细用量

```bash
# 当月用量
ds-check usage

# 指定月份
ds-check usage -m 4 -y 2026

# 列出当月使用的所有模型
ds-check usage -M list

# 为每个模型单独渲染一张表格
ds-check usage -M all

# 按模型筛选（子串匹配）
ds-check usage -M v4-pro
ds-check usage -M flash
```

### JSON 输出

```bash
ds-check summary --json
ds-check usage --json -m 5
ds-check usage --json -M flash
```

### 切换语言

```bash
ds-check --locale zh_CN summary
ds-check --locale ja_JP usage
```

未指定时，自动从 `LANG` 环境变量检测语言。

支持的语言：`zh_CN`、`zh_TW`、`en_US`、`ja_JP`。

### ASCII 渲染模式

```bash
# 纯 ASCII 表格，无 Unicode 边框和颜色
DSCHECK_RENDER=ascii ds-check summary
DSCHECK_RENDER=ascii ds-check usage
```

ASCII 模式强制使用英文标签，避免输出 Unicode 字符。

## 环境变量

| 变量 | 取值 | 说明 |
|---|---|---|
| `DSCHECK_MOCK` | `1` | 使用模拟数据，不发起网络请求 |
| `DSCHECK_RENDER` | `ascii` / `unicode` | 输出样式（默认：`unicode`） |
| `DSCHECK_LOCALE` | `zh_CN`、`zh_TW`、`en_US`、`ja_JP` | 设置默认语言 |
| `LANG` | 如 `zh_CN.UTF-8` | 未指定 `--locale` 时自动检测 |

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
