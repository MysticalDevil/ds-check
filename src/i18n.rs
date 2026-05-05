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
                "token_saved" => "Token 已保存到 {}",
                "use_help" => "未指定子命令，使用 ds-check --help 查看可用命令",
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
                "config_load_error" => "加载配置文件失败: {}",
                _ => key,
            },
            Self::ZhTW => match key {
                "no_token" => "未檢測到登錄憑據",
                "auth_hint" => "請先登錄: ds-check auth <你的Token>",
                "enter_token" => "請輸入 DeepSeek API Token: ",
                "token_help" => "Token 獲取地址: https://platform.deepseek.com/api_keys",
                "auth_success" => "登錄成功: {}",
                "invalid_token" => "Token 驗證失敗，請檢查後重試",
                "token_saved" => "Token 已儲存至 {}",
                "use_help" => "未指定子命令，使用 ds-check --help 查看可用命令",
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
                "config_load_error" => "載入設定檔失敗: {}",
                _ => key,
            },
            Self::EnUS => match key {
                "no_token" => "No login credentials found",
                "auth_hint" => "Please login first: ds-check auth <your-token>",
                "enter_token" => "Enter your DeepSeek API token: ",
                "token_help" => "Get your token at: https://platform.deepseek.com/api_keys",
                "auth_success" => "Logged in as: {}",
                "invalid_token" => "Token validation failed, please check and retry",
                "token_saved" => "Token saved to {}",
                "use_help" => "No subcommand specified. Use ds-check --help for available commands",
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
                "config_load_error" => "Failed to load config: {}",
                _ => key,
            },
            Self::JaJP => match key {
                "no_token" => "ログイン情報が見つかりません",
                "auth_hint" => "まずログインしてください: ds-check auth <トークン>",
                "enter_token" => "DeepSeek API トークンを入力してください: ",
                "token_help" => "トークンの取得先: https://platform.deepseek.com/api_keys",
                "auth_success" => "ログイン成功: {}",
                "invalid_token" => "トークンの検証に失敗しました。確認して再試行してください",
                "token_saved" => "トークンを {} に保存しました",
                "use_help" => {
                    "サブコマンドが指定されていません。ds-check --help で利用可能なコマンドを確認してください"
                }
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
                "config_load_error" => "設定の読み込みに失敗: {}",
                _ => key,
            },
        }
    }
}
