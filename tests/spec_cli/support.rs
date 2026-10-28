use assert_cmd::Command;

pub(super) fn yamark() -> Command {
    Command::cargo_bin("yamark").unwrap()
}

pub(super) fn run_stdin(args: &[&str], stdin: &str) -> (i32, String, String) {
    let output = yamark().args(args).write_stdin(stdin).output().unwrap();
    (
        output.status.code().unwrap_or(1),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
