# Coding Challenges

Rust reimplementations of familiar command-line tools, built one step at a time, in a single Cargo workspace.

## Overview

Each challenge is a separate crate in the workspace, named after its challenge number. The `wc` and JSON parser challenges follow the step-by-step briefs from [Coding Challenges](https://codingchallenges.fyi/).

| Crate | Directory | Binary | What it is | Status |
| --- | --- | --- | --- | --- |
| `day001-wc` | `001-wc/` | `ccwc` | A clone of `wc` | Complete |
| `day002-json` | `002-json/` | `day002-json` | A JSON parser | Lexer complete; parser not started |
| `day120-md5` | `120-md5/` | `day120-md5` | A clone of `md5sum`, with SHA and HMAC support, with every hash written from scratch | Complete |

## Challenges

### 001: `wc` (`ccwc`)

Counts lines, words, bytes and characters in files or standard input.

- `-c, --bytes`: byte count
- `-l, --lines`: line count (newline characters)
- `-w, --words`: word count (runs of non-whitespace)
- `-m, --chars`: character count, decoding the input as UTF-8
- With no option, prints lines, words and bytes, as `wc` does.
- With no file arguments, reads standard input.

```sh
cargo run -p day001-wc -- -l 001-wc/tests/fixtures/test.txt
cat 001-wc/tests/fixtures/test.txt | cargo run -p day001-wc -- -w
cargo run -p day001-wc -- 001-wc/tests/fixtures/test.txt
```

### 002: JSON parser

So far this is a lexer (`002-json/src/lexer.rs`) that turns a JSON document into tokens (braces, brackets, colons, commas, strings, numbers, booleans and `null`), each tagged with its line, column and byte index. It handles:

- string escapes, including `\u` escapes and UTF-16 surrogate pairs
- number validation for integer, fraction and exponent forms
- whitespace and control-character rules following RFC 8259

Lexing errors report where they happened and why: an unexpected token, a bad number, a bad escape character, a control character in a string, an unterminated string or a bad hex escape.

`parser.rs` is empty and `main.rs` only prints a placeholder, so there is nothing useful to run yet; the lexer is exercised through its unit tests.

### 120: `md5sum`

A reimplementation of `md5sum`. MD5 and the SHA family are implemented by hand in `120-md5/src/process/`, including message padding, rather than taken from a crypto library.

```
Usage: day120-md5 [OPTIONS] [FILES]...

  -a, --algorithm <ALGORITHM>  md5 (default), sha1, sha256, sha384, sha512
  -b, --binary                 mark output lines with '*' (binary mode)
  -c, --check                  read checksums from the files and check them
      --quiet                  with --check, don't print OK for each verified file
      --status                 print nothing; report the result through the exit code
      --tag                    print BSD-style output, e.g. "MD5 (-) = ..."
      --hmac <KEY>             compute an HMAC with KEY (MD5 and SHA-256 only)
```

A file name of `-`, or no file at all, reads standard input. Messages and warnings for unreadable files, mismatches and malformed checksum lines follow GNU `md5sum`, and the exit code is non-zero if any file is missing or fails its check.

```sh
cargo run -p day120-md5 -- 120-md5/tests/fixtures/test_file.txt
cargo run -p day120-md5 -- -a sha256 120-md5/tests/fixtures/test_file.txt
echo -n test | cargo run -p day120-md5 -- -a sha256 --hmac test
cargo run -p day120-md5 -- --tag 120-md5/tests/fixtures/test_file.txt
```

`--check` reads paths relative to the current directory, so run it from inside `120-md5/` to use the bundled checksum files:

```sh
cd 120-md5
cargo run -- -c tests/fixtures/checksums.txt
cargo run -- -a sha512 -c tests/fixtures/checksums_sha512.txt
```

## Tech stack

- Rust, 2024 edition, in a Cargo workspace (resolver 3)
- [`clap`](https://crates.io/crates/clap) (derive) for argument parsing
- [`regex`](https://crates.io/crates/regex) for word counting and checksum-line parsing
- [`bitvec`](https://crates.io/crates/bitvec), [`hex`](https://crates.io/crates/hex) and [`libc`](https://crates.io/crates/libc) in the hashing tool
- [`assert_cmd`](https://crates.io/crates/assert_cmd) and [`predicates`](https://crates.io/crates/predicates) for CLI integration tests

## Getting started

Requires a Rust toolchain that supports the 2024 edition (Rust 1.85 or later). The `md5sum` tool uses `libc::dup2` for `--status`, so it targets Unix-like systems.

```sh
cargo build --workspace
```

## Project structure

```
Cargo.toml                 # workspace definition
001-wc/
├── src/main.rs            # ccwc: arguments, counting and tests
└── tests/fixtures/        # sample text files
002-json/
├── src/lexer.rs           # JSON lexer and its unit tests
├── src/parser.rs          # empty, for the parser stage
└── tests/fixtures/        # sample JSON
120-md5/
├── rfc1321.txt            # the MD5 specification (RFC 1321), for reference
├── src/main.rs            # CLI, hashing and checksum verification
├── src/process.rs         # ComputeHash and Hmac traits, HMAC key padding
├── src/process/           # md5, sha1, sha256, sha384, sha512 and padding
└── tests/                 # integration tests and fixture files
```

## Tests

```sh
cargo test --workspace
```

- `001-wc`: unit tests for each count, plus end-to-end tests that run the binary through `cargo run` on files and on piped input.
- `002-json`: lexer unit tests covering full documents, positions, escapes, surrogate pairs, number formats and each error type.
- `120-md5`: padding unit tests, test vectors for each algorithm, and integration tests in `tests/integration.rs` for standard input, multiple files, `--binary`, `--check` (including malformed lines and missing files), `--quiet`, `--status`, `--tag` and HMAC.

## Status and limitations

- The JSON parser stage has not been started.
- `--hmac` only applies to MD5 and SHA-256, is ignored for the other algorithms, and is not used with `--check`.
- `ccwc` reads files as UTF-8 text, so it reports an error on files that are not valid UTF-8.

## Acknowledgements

- [Coding Challenges](https://codingchallenges.fyi/) by John Crickett, for the `wc` and JSON parser briefs.
- [RFC 1321](https://www.rfc-editor.org/rfc/rfc1321), the MD5 message-digest algorithm.
