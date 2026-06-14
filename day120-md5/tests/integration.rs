use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn test_md5_using_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "d41d8cd98f00b204e9800998ecf8427e  -",
        ));
}

#[test]
fn test_md5_one_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.arg("tests/test_file.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "cfa563b916ab0abd03659d0c40aef995  tests/test_file.txt",
        ));
}
