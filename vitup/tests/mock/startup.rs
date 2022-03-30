use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;

#[test]
pub fn start_mock() {
    let config_file_path = "example/mock/config.yaml";
    let mut cmd = Command::cargo_bin("vitup").unwrap();
    let mut mock_process = cmd
        .arg("start")
        .arg("mock")
        .arg("--config")
        .arg(&config_file_path)
        .spawn()
        .unwrap();
    let configuration = vitup::mode::mock::read_config(&config_file_path).unwrap();

    let request = reqwest::blocking::Client::new()
        .get(&format!(
            "http://localhost:{}/api/health",
            configuration.port
        ))
        .send();

    mock_process.kill().unwrap();
    assert_eq!(request.unwrap().status(), 200);
}
