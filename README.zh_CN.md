# ds-check

用于查询 DeepSeek 开放平台用量、余额和 API 消费的 CLI 工具。

## 功能特性

- 查看账户余额和月度消费
- 追踪 API 请求次数和 Token 用量
- 按模型和 Token 类型查看每日用量明细
- 多语言支持：zh_CN、zh_TW、en_US、ja_JP
- JSON 输出模式，便于脚本集成
- 跨平台（Linux、macOS、Windows）

## 安装

```bash
cargo install --path .
```

## 用法

### 查看余额和用量

```bash
ds-check
```

输出示例：

```
============ DeepSeek 使用量 ============
     用户: nickname
     余额: 121.75 CNY
 本月消费: 10.30 CNY
API 请求: 785
   Tokens: 82.56M
========================================
```

### 登录认证

```bash
# 直接传入 Token
ds-check auth sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 交互式输入
ds-check auth
```

Token 和用户信息存储在 `$XDG_CONFIG_HOME/ds-check/auth.json`。

### 查看详细用量

```bash
# 当月用量
ds-check usage

# 指定月份
ds-check usage -m 4 -y 2026

# 按模型筛选
ds-check usage -M v4-pro
```

### JSON 输出

```bash
ds-check --json
ds-check usage --json -m 5
```

### 切换语言

```bash
ds-check --locale zh_CN
ds-check --locale ja_JP
```

未指定时，自动从 `LANG` 环境变量检测语言。

支持的语言：`zh_CN`、`zh_TW`、`en_US`、`ja_JP`。

## 许可证

MIT
