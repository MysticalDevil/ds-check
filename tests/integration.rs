use std::process::Command;

fn ds_check() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ds-check"));
    cmd.env("DSCHECK_MOCK", "1");
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
    let output = ds_check().arg("auth").arg("fake-token").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MockUser"));
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
