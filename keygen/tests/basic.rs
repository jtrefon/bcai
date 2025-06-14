use assert_cmd::Command;
use keygen_lib::KeypairJson;

#[test]
fn generates_keypair() {
    let assert = Command::cargo_bin("keygen").unwrap().assert().success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let pair: KeypairJson = serde_json::from_str(&out).unwrap();
    assert_eq!(pair.public.len(), 64);
    assert_eq!(pair.secret.len(), 64);
}
