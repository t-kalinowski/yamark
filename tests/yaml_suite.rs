use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn yaml_test_suite_conformance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/yaml-test-suite/data");
    if !root.exists() {
        eprintln!(
            "skipping yaml-test-suite conformance; fixture data missing at {}",
            root.display()
        );
        return;
    }

    let mut cases = Vec::new();
    collect_cases(&root, &mut cases);
    cases.sort();
    if let Ok(filter) = std::env::var("YAMARK_SUITE_CASE") {
        cases.retain(|case| {
            case.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .contains(&filter)
        });
    }

    let mut failures = Vec::new();
    for case in cases {
        if let Err(err) = check_case(&case) {
            failures.push(format!(
                "{}: {err}",
                case.strip_prefix(&root).unwrap().display()
            ));
            if failures.len() >= 40 {
                break;
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} yaml-test-suite cases failed:\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

fn collect_cases(path: &Path, cases: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_cases(&path, cases);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("in.yaml") {
            cases.push(path.parent().unwrap().to_owned());
        }
    }
}

fn check_case(case: &Path) -> Result<(), String> {
    let input_path = case.join("in.yaml");
    let input = std::fs::read_to_string(&input_path).map_err(|err| err.to_string())?;

    if case.join("error").exists() {
        if normalize_yaml_file(&input_path).is_ok() {
            return Err("py-yaml12 accepted an error-marked suite case".to_owned());
        }
        return Ok(());
    }

    let before = normalize_yaml_file(&input_path)?;
    let formatted =
        format_with_cli(&input).map_err(|err| format!("Yamark rejected valid input: {err}"))?;

    let formatted_file = tempfile::NamedTempFile::new().map_err(|err| err.to_string())?;
    std::fs::write(formatted_file.path(), &formatted).map_err(|err| err.to_string())?;
    let after = normalize_yaml_file(formatted_file.path())?;
    if before != after {
        return Err(format!(
            "formatted YAML changed semantic value\ninput:     {before}\nformatted: {after}"
        ));
    }

    if case.join("in.json").exists() {
        let expected = normalize_json_file(&case.join("in.json"))?;
        if before != expected {
            return Err(format!(
                "input YAML does not match in.json\ninput: {before}\njson:  {expected}"
            ));
        }
        if after != expected {
            return Err(format!(
                "formatted YAML does not match in.json\nformatted: {after}\njson:      {expected}"
            ));
        }
    }

    let reformatted = format_with_cli(&formatted)
        .map_err(|err| format!("Yamark rejected formatted output: {err}"))?;
    if reformatted != formatted {
        return Err("formatted output is not idempotent".to_owned());
    }

    Ok(())
}

fn format_with_cli(input: &str) -> Result<String, String> {
    let temp = tempfile::tempdir().map_err(|err| err.to_string())?;
    let path = temp.path().join("case.yaml");
    std::fs::write(&path, input).map_err(|err| err.to_string())?;

    let output = Command::new(env!("CARGO_BIN_EXE_yamark"))
        .arg("format")
        .arg(&path)
        .output()
        .map_err(|err| format!("failed to run yamark: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "yamark format failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::read_to_string(&path).map_err(|err| err.to_string())
}

fn normalize_yaml_file(path: &Path) -> Result<String, String> {
    normalize_with_python("yaml", path)
}

fn normalize_json_file(path: &Path) -> Result<String, String> {
    normalize_with_python("json", path)
}

fn normalize_with_python(mode: &str, path: &Path) -> Result<String, String> {
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("external-tests/support/yaml_suite_value.py");
    let output = Command::new("python3")
        .arg(script)
        .arg(mode)
        .arg(path)
        .output()
        .map_err(|err| format!("failed to run python3: {err}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    Ok(String::from_utf8(output.stdout)
        .map_err(|err| err.to_string())?
        .trim_end()
        .to_owned())
}
