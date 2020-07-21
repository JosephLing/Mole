use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process;

#[test]
pub fn no_subcommand() {
    process::Command::cargo_bin("mole")
        .unwrap()
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("One of the following subcommands must be present:")
                .from_utf8(),
        );
}

#[test]
pub fn build_no_path() {
    process::Command::cargo_bin("mole")
        .unwrap()
        .args(&["build"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("The system cannot find the path specified.").from_utf8());
}
