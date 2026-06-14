use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        let _ = Command::new(env!("CARGO_BIN_EXE_metrix"))
            .env("METRIX_MOCK", "1")
            .env("XDG_CONFIG_HOME", "/tmp/metrix-test-config")
            .env("XDG_CACHE_HOME", "/tmp/metrix-test-cache")
            .arg("auth")
            .arg("fake-token")
            .output();

        // Write a mock pricing.json into the test cache directory
        let cache_dir = std::path::PathBuf::from("/tmp/metrix-test-cache/metrix");
        let _ = std::fs::create_dir_all(&cache_dir);
        let pricing_json = r#"{
  "currency": "CNY",
  "unit": "per 1M tokens",
  "note": "Test pricing note",
  "models": [
    {
      "model": "deepseek-v4-flash",
      "input_cache_hit": "0.02",
      "input_cache_miss": "1",
      "output": "2"
    },
    {
      "model": "deepseek-v4-pro",
      "input_cache_hit": "0.025",
      "input_cache_miss": "3",
      "output": "6"
    }
  ]
}"#;
        let _ = std::fs::write(cache_dir.join("pricing.json"), pricing_json);
    });
}

fn metrix() -> Command {
    init_test_env();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    cmd.env("METRIX_MOCK", "1");
    cmd.env("XDG_CONFIG_HOME", "/tmp/metrix-test-config");
    cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    cmd
}

#[test]
fn test_no_subcommand_exits_with_help() {
    let output = metrix().output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_summary_outputs_balance() {
    let output = metrix().arg("summary").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("121.76") || stdout.contains("CNY"));
}

#[test]
fn test_summary_json_output() {
    let output = metrix().arg("summary").arg("--json").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"balance\""));
    assert!(stdout.contains("\"monthly_cost\""));
}

#[test]
fn test_usage_outputs_table() {
    let output = metrix().arg("usage").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Title shows first model; table body includes mixed model data
    assert!(stdout.contains("deepseek-v4-pro"));
    // Flash model data (640.00K from day 1) appears in the combined table
    assert!(stdout.contains("640.00K"));
}

#[test]
fn test_usage_json_output() {
    let output = metrix().arg("usage").arg("--json").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"model\""));
    assert!(stdout.contains("\"cost\""));
}

#[test]
fn test_models_lists_all_models() {
    let output = metrix().arg("models").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<&str> = stdout.lines().collect();
    assert!(models.contains(&"deepseek-v4-pro"));
    assert!(models.contains(&"deepseek-v4-flash"));
    // Should show apikey hint on stderr when no api_key is configured
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("apikey") || stderr.contains("API Key"));
}

#[test]
fn test_models_with_api_key() {
    // Use an isolated config directory to avoid parallel test interference
    let temp_config =
        std::env::temp_dir().join(format!("metrix-apikey-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_config);

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    cmd.env("METRIX_MOCK", "1");
    cmd.env("XDG_CONFIG_HOME", &temp_config);
    cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");

    // First auth with token
    let auth_output = cmd.arg("auth").arg("fake-token").output().unwrap();
    assert!(
        auth_output.status.success(),
        "auth failed: {}",
        String::from_utf8_lossy(&auth_output.stderr)
    );

    // Then set API Key
    let mut cmd_apikey = Command::new(env!("CARGO_BIN_EXE_metrix"));
    cmd_apikey.env("METRIX_MOCK", "1");
    cmd_apikey.env("XDG_CONFIG_HOME", &temp_config);
    cmd_apikey.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let apikey_output = cmd_apikey.arg("apikey").arg("sk-test123").output().unwrap();
    assert!(
        apikey_output.status.success(),
        "apikey failed: {}",
        String::from_utf8_lossy(&apikey_output.stderr)
    );

    // Then models should use the API Key route
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_metrix"));
    cmd2.env("METRIX_MOCK", "1");
    cmd2.env("XDG_CONFIG_HOME", &temp_config);
    cmd2.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let output = cmd2.arg("models").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("deepseek-v4-flash"));
    assert!(stdout.contains("deepseek-v4-pro"));
    // Should NOT show apikey hint when api_key is configured
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("apikey") && !stderr.contains("API Key"));

    let _ = std::fs::remove_dir_all(&temp_config);
}

#[test]
fn test_kimi_summary_and_models_mock_mode() {
    let temp_config = std::env::temp_dir().join(format!("metrix-kimi-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_config);

    let mut auth_cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    auth_cmd.env("METRIX_MOCK", "1");
    auth_cmd.env("XDG_CONFIG_HOME", &temp_config);
    auth_cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let auth_output = auth_cmd
        .arg("--provider")
        .arg("kimi")
        .arg("auth")
        .arg("--api-key")
        .arg("sk-kimi-test")
        .output()
        .unwrap();
    assert!(
        auth_output.status.success(),
        "kimi auth failed: {}",
        String::from_utf8_lossy(&auth_output.stderr)
    );

    let mut summary_cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    summary_cmd.env("METRIX_MOCK", "1");
    summary_cmd.env("XDG_CONFIG_HOME", &temp_config);
    summary_cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let summary = summary_cmd
        .arg("--provider")
        .arg("kimi")
        .arg("summary")
        .arg("--json")
        .output()
        .unwrap();
    assert!(summary.status.success());
    let stdout = String::from_utf8_lossy(&summary.stdout);
    assert!(stdout.contains("49.58894"));

    let mut models_cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    models_cmd.env("METRIX_MOCK", "1");
    models_cmd.env("XDG_CONFIG_HOME", &temp_config);
    models_cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let models = models_cmd
        .arg("--provider")
        .arg("kimi")
        .arg("models")
        .output()
        .unwrap();
    assert!(models.status.success());
    let stdout = String::from_utf8_lossy(&models.stdout);
    assert!(stdout.contains("kimi-k2.5"));

    let _ = std::fs::remove_dir_all(&temp_config);
}

#[test]
fn test_bigmodel_summary_is_explicitly_unsupported() {
    let temp_config =
        std::env::temp_dir().join(format!("metrix-bigmodel-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_config);

    let mut auth_cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    auth_cmd.env("METRIX_MOCK", "1");
    auth_cmd.env("XDG_CONFIG_HOME", &temp_config);
    auth_cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let auth_output = auth_cmd
        .arg("--provider")
        .arg("bigmodel")
        .arg("auth")
        .arg("--platform-token")
        .arg("web-token")
        .arg("--api-key")
        .arg("api-key")
        .output()
        .unwrap();
    assert!(
        auth_output.status.success(),
        "bigmodel auth failed: {}",
        String::from_utf8_lossy(&auth_output.stderr)
    );

    let mut summary_cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    summary_cmd.env("METRIX_MOCK", "1");
    summary_cmd.env("XDG_CONFIG_HOME", &temp_config);
    summary_cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");
    let summary = summary_cmd
        .arg("--provider")
        .arg("bigmodel")
        .arg("summary")
        .output()
        .unwrap();
    assert!(!summary.status.success());
    let stderr = String::from_utf8_lossy(&summary.stderr);
    assert!(stderr.contains("bigmodel"));
    assert!(stderr.contains("summary"));

    let _ = std::fs::remove_dir_all(&temp_config);
}

#[test]
fn test_usage_model_filter() {
    let output = metrix()
        .arg("usage")
        .arg("--model")
        .arg("flash")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("deepseek-v4-flash"));
    assert!(!stdout.contains("deepseek-v4-pro"));
}

#[test]
fn test_usage_model_filter_no_match() {
    let output = metrix()
        .arg("usage")
        .arg("--model")
        .arg("nonexistent-model-xyz")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should print "No data available" or localized equivalent
    assert!(stdout.contains("No data") || stdout.contains("暂无数据"));
}

#[test]
fn test_auth_mock_mode() {
    // Use an isolated config directory to avoid overwriting shared auth.json
    let temp_config = std::env::temp_dir().join(format!("metrix-auth-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_config);

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_metrix"));
    cmd.env("METRIX_MOCK", "1");
    cmd.env("XDG_CONFIG_HOME", &temp_config);
    cmd.env("XDG_CACHE_HOME", "/tmp/metrix-test-cache");

    let output = cmd.arg("auth").arg("fake-token").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MockUser"));

    let _ = std::fs::remove_dir_all(&temp_config);
}

#[test]
fn test_ascii_render_summary() {
    let output = metrix()
        .env("METRIX_RENDER", "ascii")
        .arg("summary")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ASCII mode uses = for header underline
    assert!(stdout.contains("==="));
}

#[test]
fn test_ascii_render_usage() {
    let output = metrix()
        .env("METRIX_RENDER", "ascii")
        .arg("usage")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ASCII usage table uses pipe delimiters
    assert!(stdout.contains("|"));
    assert!(stdout.contains("deepseek-v4-pro"));
}

#[test]
fn test_locale_en_override() {
    let output = metrix()
        .arg("summary")
        .arg("--locale")
        .arg("en_US")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Balance"));
    assert!(stdout.contains("Monthly Cost"));
}

#[test]
fn test_locale_zh_override() {
    let output = metrix()
        .arg("summary")
        .arg("--locale")
        .arg("zh_CN")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("充值余额"));
    assert!(stdout.contains("本月消费"));
}

#[test]
fn test_price_shows_models() {
    let output = metrix().arg("price").output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("deepseek-v4-flash"));
    assert!(stdout.contains("deepseek-v4-pro"));
    assert!(stdout.contains("0.02元") || stdout.contains("0.02"));
}

#[test]
fn test_price_json_output() {
    let output = metrix().arg("price").arg("--json").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"currency\""));
    assert!(stdout.contains("\"models\""));
    assert!(stdout.contains("deepseek-v4-pro"));
}

#[test]
fn test_price_ascii_render() {
    let output = metrix()
        .env("METRIX_RENDER", "ascii")
        .arg("price")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("==="));
    assert!(stdout.contains("deepseek-v4-flash"));
}

#[test]
fn test_price_locale_en_override() {
    let output = metrix()
        .arg("price")
        .arg("--locale")
        .arg("en_US")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Model Pricing") || stdout.contains("Model"));
}

#[test]
fn test_usage_json_with_model_filter() {
    let output = metrix()
        .arg("usage")
        .arg("--json")
        .arg("--model")
        .arg("flash")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSON array should only contain flash model entries
    assert!(stdout.contains("\"model\": \"deepseek-v4-flash\""));
    assert!(!stdout.contains("\"model\": \"deepseek-v4-pro\""));
}
