use assert_cmd::Command;
use std::fs;

fn setup() {
    let _ = fs::remove_file("jobs.json");
}

#[test]
fn post_and_list() {
    setup();
    Command::cargo_bin("jobmanager").unwrap().args(["post", "test job", "100"]).assert().success();

    let assert = Command::cargo_bin("jobmanager").unwrap().arg("list").assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(output.contains("test job"));
}
