use std::fs;
use std::path::Path;

use assert_cmd::Command;

#[derive(Default)]
struct Case {
    args: String,
    stdin: String,
    stdout: String,
    stderr: String,
    status: i32,
}

#[test]
fn cli_cases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let mut cases = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("case"))
        .collect::<Vec<_>>();
    cases.sort();

    for path in cases {
        let case = parse_case(&fs::read_to_string(&path).unwrap());
        let mut command = Command::cargo_bin("yamark").unwrap();
        command.args(split_args(&case.args));
        command.write_stdin(case.stdin);
        let assert = command.assert().code(case.status);
        let output = assert.get_output();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            case.stdout,
            "stdout mismatch in {}",
            path.display()
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            case.stderr,
            "stderr mismatch in {}",
            path.display()
        );
    }
}

fn parse_case(input: &str) -> Case {
    let mut case = Case {
        status: 0,
        ..Case::default()
    };
    let mut section = String::new();
    let mut body = String::new();

    for line in input.lines() {
        if let Some(name) = line.strip_prefix("-- ") {
            flush(&mut case, &section, &body);
            section = name.trim().to_owned();
            body.clear();
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    flush(&mut case, &section, &body);
    case
}

fn flush(case: &mut Case, section: &str, body: &str) {
    let value = body.strip_suffix('\n').unwrap_or(body);
    match section {
        "args" => case.args = value.to_owned(),
        "stdin" => case.stdin = body.to_owned(),
        "stdout" => case.stdout = body.to_owned(),
        "stderr" => case.stderr = body.to_owned(),
        "status" => case.status = value.trim().parse().unwrap(),
        "" => {}
        other => panic!("unknown case section {other}"),
    }
}

fn split_args(input: &str) -> Vec<String> {
    input.split_whitespace().map(ToOwned::to_owned).collect()
}
