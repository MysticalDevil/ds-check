# DeepSeek 开放平台 - 用量/账户 API 文档

> 本文档通过抓包分析 [platform.deepseek.com](https://platform.deepseek.com) 获得，
> 记录与用量信息、余额、账单相关的后台 API。

---

## 通用说明

### 请求头

所有 API 请求均需携带以下请求头：

| Header | 值 | 说明 |
|---|---|---|
| `Authorization` | `Bearer <token>` | 登录后获取的 token |
| `x-app-version` | `20240425.0` | 客户端版本标识 |
| `Cookie` | 登录态 cookie | 含 `smidV2` 等 |

### 响应结构

所有接口统一响应格式：

```json
{
  "code": 0,
  "msg": "",
  "data": {
    "biz_code": 0,
    "biz_msg": "",
    "biz_data": { ... }
  }
}
```

- `code=0`、`biz_code=0` 表示成功
- 实际数据在 `data.biz_data` 中

### Token 获取方式

Token 在浏览器登录平台后，可从任意 API 请求头的 `Authorization: Bearer <token>` 中获取。

---

## 一、用户信息 & 登录态

### 1.1 获取当前用户信息

```
GET https://platform.deepseek.com/auth-api/v0/users/current
```

**用途**：获取当前登录用户的个人信息、余额预警设置、功能开关等。

**响应字段说明**：

| 字段路径 | 类型 | 说明 |
|---|---|---|
| `biz_data.id` | string | 用户 UUID |
| `biz_data.token` | string | 当前 API token |
| `biz_data.email` | string | 脱敏邮箱 |
| `biz_data.mobile_number` | string | 脱敏手机号 |
| `biz_data.status` | number | 账户状态 (0=正常) |
| `biz_data.id_profile.name` | string | 用户昵称 |
| `biz_data.id_profile.locale` | string | 语言地区 (`zh_CN`) |
| `biz_data.currency` | string | 货币 (`CNY`) |
| `biz_data.feature_gates.PAYMENT` | boolean | 是否开通支付 |
| `biz_data.feature_gates.ANTOM_ENABLED` | boolean | 功能开关 |
| `biz_data.balance_alert.CNY` | object | CNY 余额预警配置 |
| `biz_data.balance_alert.CNY.enabled` | boolean | 是否开启预警 |
| `biz_data.balance_alert.CNY.alert_bound` | string | 预警阈值金额 |
| `biz_data.identity_verification_id` | string | 个人认证 ID |
| `biz_data.business_verification_id` | string | 企业认证 ID（可能为 `null`） |

**响应示例**：

```json
{
  "code": 0,
  "msg": "",
  "data": {
    "biz_data": {
      "id": "fa6e1107-c8ba-4e76-bfa4-4286acc5a1c7",
      "token": "vt5CIgEM...",
      "email": "jin******81@outlook.com",
      "mobile_number": "187******50",
      "area_code": "+86",
      "status": 0,
      "id_profile": {
        "provider": "WECHAT",
        "id": "ef3e400e-e20c-4f75-84c5-18aefb22d5e5",
        "picture": "https://static.deepseek.com/user-avatar/0jy3yGoAmkqiZIqajKBtmW1T",
        "name": "可能是菠萝干也可能是萝卜片",
        "locale": "zh_CN",
        "email": null
      },
      "feature_gates": {
        "PAYMENT": true,
        "ANTOM_ENABLED": true
      },
      "currency": "CNY",
      "identity_verification_id": "671ba3ba6a324bfa8d6e104b1cb8cf38",
      "business_verification_id": null,
      "balance_alert": {
        "CNY": { "enabled": true, "alert_bound": "5" },
        "USD": { "enabled": false, "alert_bound": "1" }
      }
    }
  }
}
```

---

## 二、账户余额 & 用量摘要

### 2.1 获取用户用量摘要

```
GET https://platform.deepseek.com/api/v0/users/get_user_summary
```

**用途**：获取账户余额、本月消费金额、本月 token 用量、token 预估余额等核心数据。
这是用量信息页面的主要数据源。

**响应字段说明**：

| 字段路径 | 类型 | 说明 |
|---|---|---|
| `biz_data.current_token` | number | 当前持有的 token 配额（赠送） |
| `biz_data.monthly_usage` | string | 本月已用 token 总数 |
| `biz_data.monthly_token_usage` | string | 本月已用 token 总数（同上） |
| `biz_data.total_usage` | number | 历史总用量 |
| `biz_data.total_available_token_estimation` | string | 余额可购买的预估 token 数 |
| `biz_data.normal_wallets[]` | array | 正常充值钱包余额 |
| `biz_data.normal_wallets[].currency` | string | 货币类型 (`CNY`) |
| `biz_data.normal_wallets[].balance` | string | 余额（高精度字符串） |
| `biz_data.normal_wallets[].token_estimation` | string | 余额可购买的预估 token 数 |
| `biz_data.bonus_wallets[]` | array | 赠送钱包余额 |
| `biz_data.monthly_costs[]` | array | 本月消费 |
| `biz_data.monthly_costs[].currency` | string | 货币类型 |
| `biz_data.monthly_costs[].amount` | string | 消费金额（高精度字符串） |

**响应示例**：

```json
{
  "code": 0,
  "msg": "",
  "data": {
    "biz_data": {
      "current_token": 10000000,
      "monthly_usage": "106725664",
      "total_usage": 0,
      "normal_wallets": [
        {
          "currency": "CNY",
          "balance": "118.9975460400000000",
          "token_estimation": "39665848"
        }
      ],
      "bonus_wallets": [
        {
          "currency": "CNY",
          "balance": "0",
          "token_estimation": "0"
        }
      ],
      "total_available_token_estimation": "39665848",
      "monthly_costs": [
        {
          "currency": "CNY",
          "amount": "13.0635422000000000"
        }
      ],
      "monthly_token_usage": "106725664"
    }
  }
}
```

---

## 三、用量明细

### 3.1 Token 用量详情（按天/模型）

```
GET https://platform.deepseek.com/api/v0/usage/amount?month={月}&year={年}
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `month` | number | 是 | 月份 (1-12) |
| `year` | number | 是 | 年份 (如 2026) |

**用途**：获取指定月份每天的 token 用量明细，按模型和 token 类型拆分。

**Token 类型说明**：

| type | 说明 |
|---|---|
| `PROMPT_TOKEN` | 输入 prompt token（**注意**：平台 API 中该值始终返回 `"0"`，已无实际意义） |
| `PROMPT_CACHE_HIT_TOKEN` | 缓存命中的 prompt token |
| `PROMPT_CACHE_MISS_TOKEN` | 缓存未命中的 prompt token |
| `RESPONSE_TOKEN` | 模型输出 token |
| `REQUEST` | API 请求次数 |

> **输入 Token 计算方式**：由于 `PROMPT_TOKEN` 恒为 0，**实际输入 token 数 = `PROMPT_CACHE_HIT_TOKEN` + `PROMPT_CACHE_MISS_TOKEN`**。消费金额也仅与这两项及 `RESPONSE_TOKEN` 相关。

**响应结构**：

- `biz_data.total` - 该月的汇总数据（按模型）
- `biz_data.days` - 每日明细数组，**包含整月的所有日期**（即使当天无用量也会返回，各类型 amount 为 `"0"`）

**当前支持的模型**：

- `deepseek-v4-pro`
- `deepseek-v4-flash`
- `deepseek-chat & deepseek-reasoner`

**响应示例（部分）**：

```json
{
  "code": 0,
  "msg": "",
  "data": {
    "biz_data": {
      "total": [
        {
          "model": "deepseek-v4-pro",
          "usage": [
            { "type": "PROMPT_TOKEN", "amount": "0" },
            { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "103960448" },
            { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "2042255" },
            { "type": "RESPONSE_TOKEN", "amount": "722961" },
            { "type": "REQUEST", "amount": "950" }
          ]
        },
        {
          "model": "deepseek-v4-flash",
          "usage": [
            { "type": "PROMPT_TOKEN", "amount": "0" },
            { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0" },
            { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0" },
            { "type": "RESPONSE_TOKEN", "amount": "0" },
            { "type": "REQUEST", "amount": "0" }
          ]
        },
        {
          "model": "deepseek-chat & deepseek-reasoner",
          "usage": [
            { "type": "PROMPT_TOKEN", "amount": "0" },
            { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0" },
            { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0" },
            { "type": "RESPONSE_TOKEN", "amount": "0" },
            { "type": "REQUEST", "amount": "0" }
          ]
        }
      ],
      "days": [
        {
          "date": "2026-05-01",
          "data": [
            {
              "model": "deepseek-v4-pro",
              "usage": [
                { "type": "PROMPT_TOKEN", "amount": "0" },
                { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "1235072" },
                { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "259591" },
                { "type": "RESPONSE_TOKEN", "amount": "33927" },
                { "type": "REQUEST", "amount": "37" }
              ]
            },
            {
              "model": "deepseek-v4-flash",
              "usage": [
                { "type": "PROMPT_TOKEN", "amount": "0" },
                { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0" },
                { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0" },
                { "type": "RESPONSE_TOKEN", "amount": "0" },
                { "type": "REQUEST", "amount": "0" }
              ]
            },
            {
              "model": "deepseek-chat & deepseek-reasoner",
              "usage": [
                { "type": "PROMPT_TOKEN", "amount": "0" },
                { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0" },
                { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0" },
                { "type": "RESPONSE_TOKEN", "amount": "0" },
                { "type": "REQUEST", "amount": "0" }
              ]
            }
          ]
        }
        // ... 后续日期（含用量为 0 的日期）
      ]
    }
  }
}
```

### 3.2 消费金额详情（按天/模型）

```
GET https://platform.deepseek.com/api/v0/usage/cost?month={月}&year={年}
```

**参数**：同 3.1。

**用途**：获取指定月份每天的消费金额明细，按模型和 token 类型拆分。

**响应结构**：与 `usage/amount` 相同，但 `usage[].amount` 为金额（CNY 元）而非 token 数。

**响应示例（部分）**：

```json
{
  "code": 0,
  "msg": "",
  "data": {
    "biz_data": [
      {
        "total": [
          {
            "model": "deepseek-v4-pro",
            "usage": [
              { "type": "PROMPT_TOKEN", "amount": "0" },
              { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "2.5990112000000000" },
              { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "6.1267650000000000" },
              { "type": "RESPONSE_TOKEN", "amount": "4.3377660000000000" },
              { "type": "REQUEST", "amount": "0" }
            ]
          },
          {
            "model": "deepseek-v4-flash",
            "usage": [
              { "type": "PROMPT_TOKEN", "amount": "0" },
              { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0" },
              { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0" },
              { "type": "RESPONSE_TOKEN", "amount": "0" },
              { "type": "REQUEST", "amount": "0" }
            ]
          },
          {
            "model": "deepseek-chat & deepseek-reasoner",
            "usage": [
              { "type": "PROMPT_TOKEN", "amount": "0" },
              { "type": "PROMPT_CACHE_HIT_TOKEN", "amount": "0" },
              { "type": "PROMPT_CACHE_MISS_TOKEN", "amount": "0" },
              { "type": "RESPONSE_TOKEN", "amount": "0" },
              { "type": "REQUEST", "amount": "0" }
            ]
          }
        ],
        "days": [ ... ]
      }
    ]
  }
}
```

**注意**：此接口的 `data.biz_data` 是个数组（外层包裹了一层），需要取 `[0]`。

---

## 四、充值账单

### 4.1 获取充值/交易记录

```
GET https://platform.deepseek.com/auth-api/v0/users/get_all_invoice
```

**用途**：获取全部充值记录和赠送记录。

**响应字段说明**：

| 字段路径 | 类型 | 说明 |
|---|---|---|
| `biz_data.invoices.payment_orders[]` | array | 充值订单列表 |
| `payment_orders[].payment_order_id` | string | 订单编号 |
| `payment_orders[].amount` | string | 充值金额 |
| `payment_orders[].currency` | string | 货币类型 (`CNY`) |
| `payment_orders[].payment_order_status` | string | 订单状态 (`SUCCESS` 成功) |
| `payment_orders[].paid_at` | string | 支付时间 (ISO 8601) |
| `payment_orders[].inserted_at` | string | 创建时间 (ISO 8601) |
| `payment_orders[].updated_at` | string | 更新时间 (ISO 8601) |
| `biz_data.invoices.bonus_orders[]` | array | 赠送记录列表 |

**响应示例**：

```json
{
  "code": 0,
  "msg": "",
  "data": {
    "biz_data": {
      "invoices": {
        "payment_orders": [
          {
            "payment_order_id": "cmbUnionPay1777776646411gZel",
            "amount": "50",
            "currency": "CNY",
            "payment_order_status": "SUCCESS",
            "paid_at": "2026-05-03T02:51:01+00:00",
            "inserted_at": "2026-05-03T02:50:46.652709+00:00",
            "updated_at": "2026-05-03T02:51:01.585066+00:00"
          }
        ],
        "bonus_orders": []
      }
    }
  }
}
```

---

## 五、其他

### 5.1 客户端配置

```
GET https://platform.deepseek.com/api/v0/client/settings?did={设备ID}
GET https://platform.deepseek.com/api/v0/client/settings?scope=banner
```

**用途**：获取前端客户端配置（如运营 banner 等）。

---

## API 接口汇总表

| 类别 | 接口 | 方法 | 用途 |
|---|---|---|---|
| 认证 | `/auth-api/v0/users/current` | GET | 当前用户信息 & Token |
| 摘要 | `/api/v0/users/get_user_summary` | GET | 余额、本月消费、Token 用量摘要 |
| 用量 | `/api/v0/usage/amount?month=&year=` | GET | Token 用量明细（按天/模型） |
| 用量 | `/api/v0/usage/cost?month=&year=` | GET | 消费金额明细（按天/模型） |
| 账单 | `/auth-api/v0/users/get_all_invoice` | GET | 充值 & 赠送记录 |
| 配置 | `/api/v0/client/settings?did=` | GET | 客户端配置 |

---

## 使用示例

### Python 示例：获取用量摘要

```python
import requests

TOKEN = "your-bearer-token-here"
BASE = "https://platform.deepseek.com"

headers = {
    "Authorization": f"Bearer {TOKEN}",
    "x-app-version": "20240425.0",
}

# 获取余额和用量摘要
resp = requests.get(f"{BASE}/api/v0/users/get_user_summary", headers=headers)
data = resp.json()["data"]["biz_data"]

balance = data["normal_wallets"][0]["balance"]
monthly_cost = data["monthly_costs"][0]["amount"]
monthly_tokens = data["monthly_token_usage"]

print(f"余额: ¥{float(balance):.2f}")
print(f"本月消费: ¥{float(monthly_cost):.2f}")
print(f"本月 Tokens: {monthly_tokens}")
```

### curl 示例：获取 Token 用量明细

```bash
curl -s \
  -H "Authorization: Bearer <TOKEN>" \
  -H "x-app-version: 20240425.0" \
  "https://platform.deepseek.com/api/v0/usage/amount?month=5&year=2026" \
  | python3 -m json.tool
```

---

> **注意**：以上 API 为 DeepSeek 开放平台内部接口，非官方公开 API，可能随时变更。
> 本文档抓取时间为 2026 年 5 月。
