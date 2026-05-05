use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        let _ = Command::new(env!("CARGO_BIN_EXE_ds-check"))
            .env("DSCHECK_MOCK", "1")
            .env("XDG_CONFIG_HOME", "/tmp/ds-check-test-config")
            .env("XDG_CACHE_HOME", "/tmp/ds-check-test-cache")
            .arg("auth")
            .arg("fake-token")
            .output();

        // Write a mock pricing.json into the test cache directory
        let cache_dir = std::path::PathBuf::from("/tmp/ds-check-test-cache/ds-check");
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

fn ds_check() -> Command {
    init_test_env();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ds-check"));
    cmd.env("DSCHECK_MOCK", "1");
    cmd.env("XDG_CONFIG_HOME", "/tmp/ds-check-test-config");
    cmd.env("XDG_CACHE_HOME", "/tmp/ds-check-test-cache");
    cmd
}

#[test]
fn test_no_subcommand_exits_with_help() {
    let output = ds_check().output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected failure, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(stdout.contains("ds-check --help") || stderr.contains("ds-check --help"));
}

#[test]
fn test_summary_outputs_balance() {
    let output = ds_check().arg("summary").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("121.76") || stdout.contains("CNY"));
}

#[test]
fn test_summary_json_output() {
    let output = ds_check().arg("summary").arg("--json").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"balance\""));
    assert!(stdout.contains("\"monthly_cost\""));
}

#[test]
fn test_usage_outputs_table() {
    let output = ds_check().arg("usage").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Title shows first model; table body includes mixed model data
    assert!(stdout.contains("deepseek-v4-pro"));
    // Flash model data (640.00K from day 1) appears in the combined table
    assert!(stdout.contains("640.00K"));
}

#[test]
fn test_usage_json_output() {
    let output = ds_check().arg("usage").arg("--json").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"model\""));
    assert!(stdout.contains("\"cost\""));
}

#[test]
fn test_models_lists_all_models() {
    let output = ds_check().arg("models").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<&str> = stdout.lines().collect();
    assert!(models.contains(&"deepseek-v4-pro"));
    assert!(models.contains(&"deepseek-v4-flash"));
    // Should show api_key hint on stderr when no api_key is configured
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("api-key") || stderr.contains("API Key"));
}

#[test]
fn test_models_with_api_key() {
    // Use an isolated config directory to avoid parallel test interference
    let temp_config =
        std::env::temp_dir().join(format!("ds-check-apikey-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_config);

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ds-check"));
    cmd.env("DSCHECK_MOCK", "1");
    cmd.env("XDG_CONFIG_HOME", &temp_config);
    cmd.env("XDG_CACHE_HOME", "/tmp/ds-check-test-cache");

    // First auth with --api-key
    let auth_output = cmd
        .arg("auth")
        .arg("fake-token")
        .arg("--api-key")
        .arg("sk-test123")
        .output()
        .unwrap();
    assert!(
        auth_output.status.success(),
        "auth failed: {}",
        String::from_utf8_lossy(&auth_output.stderr)
    );

    // Then models should use the API Key route
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_ds-check"));
    cmd2.env("DSCHECK_MOCK", "1");
    cmd2.env("XDG_CONFIG_HOME", &temp_config);
    cmd2.env("XDG_CACHE_HOME", "/tmp/ds-check-test-cache");
    let output = cmd2.arg("models").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("deepseek-v4-flash"));
    assert!(stdout.contains("deepseek-v4-pro"));
    // Should NOT show api_key hint when api_key is configured
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("api-key"));

    let _ = std::fs::remove_dir_all(&temp_config);
}

#[test]
fn test_usage_model_filter() {
    let output = ds_check()
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
    let output = ds_check()
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
    let temp_config =
        std::env::temp_dir().join(format!("ds-check-auth-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_config);

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ds-check"));
    cmd.env("DSCHECK_MOCK", "1");
    cmd.env("XDG_CONFIG_HOME", &temp_config);
    cmd.env("XDG_CACHE_HOME", "/tmp/ds-check-test-cache");

    let output = cmd.arg("auth").arg("fake-token").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MockUser"));

    let _ = std::fs::remove_dir_all(&temp_config);
}

#[test]
fn test_ascii_render_summary() {
    let output = ds_check()
        .env("DSCHECK_RENDER", "ascii")
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
    let output = ds_check()
        .env("DSCHECK_RENDER", "ascii")
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
    let output = ds_check()
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
    let output = ds_check()
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
    let output = ds_check().arg("price").output().unwrap();
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
    let output = ds_check().arg("price").arg("--json").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"currency\""));
    assert!(stdout.contains("\"models\""));
    assert!(stdout.contains("deepseek-v4-pro"));
}

#[test]
fn test_price_ascii_render() {
    let output = ds_check()
        .env("DSCHECK_RENDER", "ascii")
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
    let output = ds_check()
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
    let output = ds_check()
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
