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
    cmd.arg("tests/fixtures/test_file.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "cfa563b916ab0abd03659d0c40aef995  tests/fixtures/test_file.txt",
        ));
}

#[test]
fn test_md5_two_files() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "tests/fixtures/test_file.txt",
        "tests/fixtures/test_file2.txt",
    ])
    .assert()
    .success()
    .stdout(predicate::eq(
        "cfa563b916ab0abd03659d0c40aef995  tests/fixtures/test_file.txt\n\
            2465ade3580e8edbcf5ed56ed8e1da0c  tests/fixtures/test_file2.txt\n",
    ));
}

#[test]
fn test_md5_two_files_and_one_file_that_does_not_exist() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "tests/fixtures/test_file.txt",
        "does-not-exist.txt",
        "tests/fixtures/test_file2.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "cfa563b916ab0abd03659d0c40aef995  tests/fixtures/test_file.txt\n\
        2465ade3580e8edbcf5ed56ed8e1da0c  tests/fixtures/test_file2.txt\n",
    ))
    .stderr(predicate::eq(
        "day120-md5: does-not-exist.txt: No such file or directory\n",
    ));
}

#[test]
fn test_md5_one_file_and_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/fixtures/test_file.txt", "-"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995  tests/fixtures/test_file.txt\n\
            d41d8cd98f00b204e9800998ecf8427e  -\n",
        ));
}

#[test]
fn test_md5_binary_flag() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/fixtures/test_file.txt", "-b"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995 *tests/fixtures/test_file.txt\n",
        ));

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["tests/fixtures/test_file.txt", "--binary"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cfa563b916ab0abd03659d0c40aef995 *tests/fixtures/test_file.txt\n",
        ));
}

#[test]
fn test_md5_checksum_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/fixtures/checksums.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_bad_checksum_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/fixtures/checksums_bad.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: FAILED\n\
            tests/fixtures/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_md5_multiple_checksum_files() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "-c",
        "tests/fixtures/checksums.txt",
        "tests/fixtures/checksums_bad.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "tests/fixtures/test_file.txt: OK\n\
        tests/fixtures/test_file2.txt: OK\n\
        tests/fixtures/test_file.txt: FAILED\n\
        tests/fixtures/test_file2.txt: FAILED\n",
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
        "tests/fixtures/checksums.txt",
        "tests/fixtures/test_file.txt",
        "tests/fixtures/checksums_bad.txt",
        "tests/fixtures/test_file2.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "tests/fixtures/test_file.txt: OK\n\
        tests/fixtures/test_file2.txt: OK\n\
        tests/fixtures/test_file.txt: FAILED\n\
        tests/fixtures/test_file2.txt: FAILED\n",
    ))
    .stderr(predicate::eq(
        "day120-md5: tests/fixtures/test_file.txt: no properly formatted checksum lines found\n\
        day120-md5: WARNING: 2 computed checksums did NOT match\n\
        day120-md5: tests/fixtures/test_file2.txt: no properly formatted checksum lines found\n",
    ));
}

#[test]
fn test_md5_checksum_file_bad_line() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "tests/fixtures/checksums2.txt"])
        .assert()
        .success()
        .stdout(predicate::eq("tests/fixtures/test_file.txt: OK\n"))
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
    cmd.args(["-c", "tests/fixtures/checksums_no_exist.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            does-not-exist: FAILED open or read\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: does-not-exist: No such file or directory\n\
                day120-md5: WARNING: 1 listed file could not be read\n",
        ));
}

#[test]
fn test_md5_checksum_from_stdin() {
    let stdin = include_str!("fixtures/checksums.txt");

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.arg("-c")
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_stdin_in_args() {
    let stdin = include_str!("fixtures/checksums.txt");

    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-c", "-"])
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_quiet_flag() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args([
        "-c",
        "--quiet",
        "tests/fixtures/checksums.txt",
        "tests/fixtures/checksums_bad.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout(predicate::eq(
        "tests/fixtures/test_file.txt: FAILED\n\
        tests/fixtures/test_file2.txt: FAILED\n",
    ))
    .stderr(predicate::eq(
        "day120-md5: WARNING: 2 computed checksums did NOT match\n",
    ));
}

#[test]
fn test_md5_quiet_flag_with_stdin() {
    let stdin = include_str!("fixtures/checksums.txt");

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
        "tests/fixtures/checksums.txt",
        "tests/fixtures/test_file.txt",
        "tests/fixtures/checksums_bad.txt",
        "tests/fixtures/test_file2.txt",
        "tests/fixtures/checksums2.txt",
    ])
    .assert()
    .failure()
    .code(1)
    .stdout("")
    .stderr("");
}

#[test]
fn test_sha256_on_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .args(["-a", "sha256"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  -\n",
        ));
}

#[test]
fn test_sha256_on_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha256", "tests/fixtures/test_file.txt"])
        .assert()
        .success()
        .stdout(predicate::eq("8ff0a1d9d8d2d2831fc702e54739bc0711d2059fefd8c269415be1eaa2f2df2c  tests/fixtures/test_file.txt\n"));
}

#[test]
fn test_sha256_checksum_on_md5_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha256", "-c", "tests/fixtures/checksums.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: FAILED\n\
            tests/fixtures/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_sha256_checksum_on_sha256_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha256", "-c", "tests/fixtures/checksums_sha256.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_sha1_on_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .args(["-a", "sha1"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "da39a3ee5e6b4b0d3255bfef95601890afd80709  -\n",
        ));
}

#[test]
fn test_sha1_on_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha1", "tests/fixtures/test_file.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "71c73b665547668841cabebb4fa5eaf57111d71b  tests/fixtures/test_file.txt\n",
        ));
}

#[test]
fn test_sha1_checksum_on_md5_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha1", "-c", "tests/fixtures/checksums.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: FAILED\n\
            tests/fixtures/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_sha1_checksum_on_sha1_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha1", "-c", "tests/fixtures/checksums_sha1.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_sha512_on_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .args(["-a", "sha512"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e  -\n",
        ));
}

#[test]
fn test_sha512_on_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha512", "tests/fixtures/test_file.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "f65cb565765485a8f8835fb742497806dde3e83c5697fad12f3bdc8c082684328d1f1f6034ebb4602dc9e1d398195528191dfe9e7af9bb619d288b6d7211ff9a  tests/fixtures/test_file.txt\n",
        ));
}

#[test]
fn test_sha512_checksum_on_md5_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha512", "-c", "tests/fixtures/checksums.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: FAILED\n\
            tests/fixtures/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_sha512_checksum_on_sha512_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha512", "-c", "tests/fixtures/checksums_sha512.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_sha384_on_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .args(["-a", "sha384"])
        .assert()
        .success()
        .stdout(predicate::eq("38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b  -\n"));
}

#[test]
fn test_sha384_on_file() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha384", "tests/fixtures/test_file.txt"])
        .assert()
        .success()
        .stdout(predicate::eq("36c4d4659e9d8bac70d263622f13150cba3064336b941d198b9e5174850ef4c9a6394fca690eaadd36f4072ccdcb3a84  tests/fixtures/test_file.txt\n"));
}

#[test]
fn test_sha384_checksum_on_md5_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha384", "-c", "tests/fixtures/checksums.txt"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: FAILED\n\
            tests/fixtures/test_file2.txt: FAILED\n",
        ))
        .stderr(predicate::eq(
            "day120-md5: WARNING: 2 computed checksums did NOT match\n",
        ));
}

#[test]
fn test_sha384_checksum_on_sha384_hashes() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.args(["-a", "sha384", "-c", "tests/fixtures/checksums_sha384.txt"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "tests/fixtures/test_file.txt: OK\n\
            tests/fixtures/test_file2.txt: OK\n",
        ));
}

#[test]
fn test_md5_tag_flag_using_stdin() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .args(["--tag"])
        .assert()
        .success()
        .stdout(predicate::eq(
            "MD5 (-) = d41d8cd98f00b204e9800998ecf8427e\n",
        ));
}

#[test]
fn test_hmac_md5() {
    let mut cmd = Command::cargo_bin("day120-md5").unwrap();
    cmd.write_stdin("")
        .args(["-a", "md5", "--hmac", "test-key"])
        .assert()
        .success()
        .stdout(predicate::eq("3726c9c0ea1ff6206d840c06ea9416ad  -\n"));
    //
}
