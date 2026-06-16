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

#[test]
fn test_md5_one_file_and_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/test_file.txt", "-"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995  tests/test_file.txt\n\
            d41d8cd98f00b204e9800998ecf8427e  -\n",
        ));
}

#[test]
fn test_md5_binary_flag() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/test_file.txt", "-b"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995 *tests/test_file.txt\n",
        ));

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/test_file.txt", "--binary"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995 *tests/test_file.txt\n",
        ));
}

#[test]
fn test_md5_checksum_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/checksums.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/test_file.txt: OK\n\
            tests/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_bad_checksum_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/checksums_bad.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/test_file.txt: FAILED\n\
            tests/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_md5_multiple_checksum_files() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/checksums.txt", "tests/checksums_bad.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/test_file.txt: OK\n\
            tests/test_file2.txt: OK\n\
            tests/test_file.txt: FAILED\n\
            tests/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_md5_multiple_checksum_files_and_bad_format() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "-c",
        "tests/checksums.txt",
        "tests/test_file.txt",
        "tests/checksums_bad.txt",
        "tests/test_file2.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "tests/test_file.txt: OK\n\
            tests/test_file2.txt: OK\n\
            tests/test_file.txt: FAILED\n\
            tests/test_file2.txt: FAILED\n",
    ))
    .stderr(predicate::eq(
        "day120-md5: tests/test_file.txt: no properly formatted checksum lines found\n\
        day120-md5: WARNING: 2 computed checksums did NOT match\n\
        day120-md5: tests/test_file2.txt: no properly formatted checksum lines found\n",
    ));
}

#[test]
fn test_md5_checksum_file_bad_line() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/checksums2.txt"])
        .assert()
        .success()
        .stdout(predicate::eq("tests/test_file.txt: OK\n"))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 1 line is improperly formatted\n",
        ));
}

#[test]
fn test_md5_checksum_file_does_not_exist() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "does-not-exist"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::eq(
            "day120-md5: does-not-exist: No such file or directory\n",
        ));
}

#[test]
fn test_md5_checksum_file_contains_file_that_does_not_exist() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/checksum_no_exist.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/test_file.txt: OK\n\
            does-not-exist: FAILED open or read\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: does-not-exist: No such file or directory\n\
                day120-md5: WARNING: 1 listed file could not be read\n",
        ));
}

#[test]
fn test_md5_checksum_from_stdin() {
    let stdin = include_str!("checksums.txt");

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.arg("-c")
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/test_file.txt: OK\n\
            tests/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_stdin_in_args() {
    let stdin = include_str!("checksums.txt");

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "-"])
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/test_file.txt: OK\n\
            tests/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_quiet_flag() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "-c",
        "--quiet",
        "tests/checksums.txt",
        "tests/checksums_bad.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "tests/test_file.txt: FAILED\n\
        tests/test_file2.txt: FAILED\n",
    ))
    .stderr(predicate::eq(
        "day120-md5: WARNING: 2 computed checksums did NOT match\n",
    ));
}

#[test]
fn test_md5_quiet_flag_with_stdin() {
    let stdin = include_str!("checksums.txt");

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["--quiet", "-c"])
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn test_md5_status_flag() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "-c",
        "--status",
        "tests/checksums.txt",
        "tests/test_file.txt",
        "tests/checksums_bad.txt",
        "tests/test_file2.txt",
        "tests/checksum2.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout("")
    .stderr("");
}
