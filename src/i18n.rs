#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locale {
    ZhCN,
    ZhTW,
    EnUS,
    JaJP,
}

impl Locale {
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.starts_with("zh_cn")
            || lower.starts_with("zh-cn")
            || lower == "zh"
            || lower.starts_with("zh-hans")
        {
            Self::ZhCN
        } else if lower.starts_with("zh_tw")
            || lower.starts_with("zh-tw")
            || lower.starts_with("zh_hk")
            || lower.starts_with("zh-hant")
        {
            Self::ZhTW
        } else if lower.starts_with("ja") || lower.starts_with("jp") {
            Self::JaJP
        } else {
            Self::EnUS
        }
    }

    pub fn detect() -> Self {
        std::env::var("LANG")
            .ok()
            .map(|lang| Self::from_str(&lang))
            .unwrap_or(Self::EnUS)
    }

    pub fn t(&self, key: &str) -> String {
        self.msg(key).to_string()
    }

    /// Note: all hard-coded translations are `&'static str`, but the signature uses
    /// `'a` so that unknown keys (fallback `_ => key`) type-check without allocation.
    fn msg<'a>(&self, key: &'a str) -> &'a str {
        match self {
            Self::ZhCN => match key {
                "no_token" => "未检测到登录凭据",
                "auth_hint" => "请先登录: ds-check auth <你的Token>",
                "enter_token" => "请输入 DeepSeek API Token: ",
                "token_help" => "Token 获取地址: https://platform.deepseek.com/api_keys",
                "auth_success" => "登录成功: {}",
                "invalid_token" => "Token 验证失败，请检查后重试",
                "auth_expired" => "登录凭证已过期或失效，请重新登录",
                "token_saved" => "Token 已保存到 {}",
                "header" => "DeepSeek 使用量",
                "balance" => "充值余额",
                "monthly_cost" => "本月消费",
                "api_requests" => "API 请求次数",
                "tokens" => "Tokens",
                "date" => "日期",
                "prompt_tokens" => "输入 Tokens",
                "cache_hit_tokens" => "缓存命中",
                "cache_miss_tokens" => "缓存未命中",
                "response_tokens" => "输出 Tokens",
                "requests" => "请求次数",
                "cost" => "费用",
                "total" => "合计",
                "no_data" => "暂无数据",
                "network_error" => "网络请求失败: {}",
                "parse_failed" => "响应解析失败",
                "empty_data" => "响应数据为空",
                "config_load_error" => "加载配置文件失败: {}",
                "currency_symbol" => "元",
                "api_key_hint" => "提示: 使用 ds-check apikey <key> 可获取全量模型列表",
                "api_key_saved" => "API Key 已保存",
                "price_header" => "模型定价",
                "pricing_not_found" => "未找到定价数据。请先运行: python3 scripts/fetch_pricing.py",
                "price_model" => "模型",
                "price_input_cache_hit" => "输入(缓存命中)",
                "price_input_cache_miss" => "输入(缓存未命中)",
                "price_output" => "输出",
                _ => key,
            },
            Self::ZhTW => match key {
                "no_token" => "未檢測到登錄憑據",
                "auth_hint" => "請先登錄: ds-check auth <你的Token>",
                "enter_token" => "請輸入 DeepSeek API Token: ",
                "token_help" => "Token 獲取地址: https://platform.deepseek.com/api_keys",
                "auth_success" => "登錄成功: {}",
                "invalid_token" => "Token 驗證失敗，請檢查後重試",
                "auth_expired" => "登錄憑證已過期或失效，請重新登錄",
                "token_saved" => "Token 已儲存至 {}",
                "header" => "DeepSeek 使用量",
                "balance" => "儲值餘額",
                "monthly_cost" => "本月消費",
                "api_requests" => "API 請求次數",
                "tokens" => "Tokens",
                "date" => "日期",
                "prompt_tokens" => "輸入 Tokens",
                "cache_hit_tokens" => "快取命中",
                "cache_miss_tokens" => "快取未命中",
                "response_tokens" => "輸出 Tokens",
                "requests" => "請求次數",
                "cost" => "費用",
                "total" => "合計",
                "no_data" => "暫無數據",
                "network_error" => "網路請求失敗: {}",
                "parse_failed" => "回應解析失敗",
                "empty_data" => "回應資料為空",
                "config_load_error" => "載入設定檔失敗: {}",
                "currency_symbol" => "元",
                "api_key_hint" => "提示: 使用 ds-check apikey <key> 可獲取全量模型列表",
                "api_key_saved" => "API Key 已儲存",
                "price_header" => "模型定價",
                "pricing_not_found" => "找不到定價資料。請先執行: python3 scripts/fetch_pricing.py",
                "price_model" => "模型",
                "price_input_cache_hit" => "輸入(快取命中)",
                "price_input_cache_miss" => "輸入(快取未命中)",
                "price_output" => "輸出",
                _ => key,
            },
            Self::EnUS => match key {
                "no_token" => "No login credentials found",
                "auth_hint" => "Please login first: ds-check auth <your-token>",
                "enter_token" => "Enter your DeepSeek API token: ",
                "token_help" => "Get your token at: https://platform.deepseek.com/api_keys",
                "auth_success" => "Logged in as: {}",
                "invalid_token" => "Token validation failed, please check and retry",
                "auth_expired" => "Authentication expired or invalid. Please login again",
                "token_saved" => "Token saved to {}",
                "header" => "DeepSeek Usage",
                "balance" => "Balance",
                "monthly_cost" => "Monthly Cost",
                "api_requests" => "API Requests",
                "tokens" => "Tokens",
                "date" => "Date",
                "prompt_tokens" => "Prompt Tokens",
                "cache_hit_tokens" => "Cache Hit",
                "cache_miss_tokens" => "Cache Miss",
                "response_tokens" => "Response Tokens",
                "requests" => "Requests",
                "cost" => "Cost",
                "total" => "Total",
                "no_data" => "No data available",
                "network_error" => "Network request failed: {}",
                "parse_failed" => "Failed to parse response",
                "empty_data" => "Empty response data",
                "config_load_error" => "Failed to load config: {}",
                "currency_symbol" => "CNY",
                "api_key_hint" => "Tip: Use ds-check apikey <key> to get full model list",
                "api_key_saved" => "API Key saved",
                "price_header" => "Model Pricing",
                "pricing_not_found" => {
                    "Pricing data not found. Run: python3 scripts/fetch_pricing.py"
                }
                "price_model" => "Model",
                "price_input_cache_hit" => "Input (Cache Hit)",
                "price_input_cache_miss" => "Input (Cache Miss)",
                "price_output" => "Output",
                _ => key,
            },
            Self::JaJP => match key {
                "no_token" => "ログイン情報が見つかりません",
                "auth_hint" => "まずログインしてください: ds-check auth <トークン>",
                "enter_token" => "DeepSeek API トークンを入力してください: ",
                "token_help" => "トークンの取得先: https://platform.deepseek.com/api_keys",
                "auth_success" => "ログイン成功: {}",
                "invalid_token" => "トークンの検証に失敗しました。確認して再試行してください",
                "auth_expired" => "ログイン情報の有効期限が切れました。再度ログインしてください",
                "token_saved" => "トークンを {} に保存しました",
                "header" => "DeepSeek 使用量",
                "balance" => "残高",
                "monthly_cost" => "今月の利用料金",
                "api_requests" => "APIリクエスト数",
                "tokens" => "トークン",
                "date" => "日付",
                "prompt_tokens" => "入力トークン",
                "cache_hit_tokens" => "キャッシュヒット",
                "cache_miss_tokens" => "キャッシュミス",
                "response_tokens" => "出力トークン",
                "requests" => "リクエスト数",
                "cost" => "費用",
                "total" => "合計",
                "no_data" => "データがありません",
                "network_error" => "ネットワークエラー: {}",
                "parse_failed" => "応答の解析に失敗",
                "empty_data" => "応答データが空です",
                "config_load_error" => "設定の読み込みに失敗: {}",
                "currency_symbol" => "元",
                "api_key_hint" => "ヒント: ds-check apikey <key> で全モデルリストを取得",
                "api_key_saved" => "APIキーを保存しました",
                "price_header" => "モデル価格",
                "pricing_not_found" => {
                    "価格データが見つかりません。実行してください: python3 scripts/fetch_pricing.py"
                }
                "price_model" => "モデル",
                "price_input_cache_hit" => "入力(キャッシュヒット)",
                "price_input_cache_miss" => "入力(キャッシュミス)",
                "price_output" => "出力",
                _ => key,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_zh_cn() {
        assert_eq!(Locale::from_str("zh_CN"), Locale::ZhCN);
        assert_eq!(Locale::from_str("zh-CN"), Locale::ZhCN);
        assert_eq!(Locale::from_str("zh"), Locale::ZhCN);
        assert_eq!(Locale::from_str("zh-hans"), Locale::ZhCN);
    }

    #[test]
    fn test_from_str_zh_tw() {
        assert_eq!(Locale::from_str("zh_TW"), Locale::ZhTW);
        assert_eq!(Locale::from_str("zh-tw"), Locale::ZhTW);
        assert_eq!(Locale::from_str("zh_hk"), Locale::ZhTW);
        assert_eq!(Locale::from_str("zh-hant"), Locale::ZhTW);
    }

    #[test]
    fn test_from_str_ja() {
        assert_eq!(Locale::from_str("ja_JP"), Locale::JaJP);
        assert_eq!(Locale::from_str("jp"), Locale::JaJP);
    }

    #[test]
    fn test_from_str_en_default() {
        assert_eq!(Locale::from_str("en_US"), Locale::EnUS);
        assert_eq!(Locale::from_str("C"), Locale::EnUS);
        assert_eq!(Locale::from_str("fr_FR"), Locale::EnUS);
        assert_eq!(Locale::from_str(""), Locale::EnUS);
    }

    #[test]
    fn test_t_known_keys() {
        let en = Locale::EnUS;
        assert_eq!(en.t("balance"), "Balance");
        assert_eq!(en.t("total"), "Total");

        let zh = Locale::ZhCN;
        assert_eq!(zh.t("balance"), "充值余额");
    }

    #[test]
    fn test_t_unknown_key_fallback() {
        let en = Locale::EnUS;
        assert_eq!(en.t("nonexistent_key"), "nonexistent_key");
    }

    #[test]
    fn test_t_dead_keys_removed() {
        // "user" and "model" were removed as dead keys
        let en = Locale::EnUS;
        assert_eq!(en.t("user"), "user"); // falls back to key itself
        assert_eq!(en.t("model"), "model");
    }
}
