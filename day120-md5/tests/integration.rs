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

#[test]
fn test_md5_two_files() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/test_file.txt", "tests/test_file2.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995  tests/test_file.txt\n\
             2465ade3580e8edbcf5ed56ed8e1da0c  tests/test_file2.txt\n",
        ));
}

#[test]
fn test_md5_two_files_and_one_file_that_does_not_exist() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "tests/test_file.txt",
        "does-not-exist.txt",
        "tests/test_file2.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "cfa563b916ab0abd03659d0c40aef995  tests/test_file.txt\n\
        2465ade3580e8edbcf5ed56ed8e1da0c  tests/test_file2.txt\n",
    ))
    .stderr(predicate::eq(
        "day120-md5: does-not-exist.txt: No such file or directory\n",
    ));
}
