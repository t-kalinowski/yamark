use assert_cmd::cargo::cargo_bin_cmd;

fn format_yaml(input: &str, width: usize) -> String {
    let output = cargo_bin_cmd!("yamark")
        .args([
            "format",
            "--stdin-file-path",
            "input.yaml",
            "--line-width",
            &width.to_string(),
            "--verify",
        ])
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn escaped_radix_strings_use_the_same_layout_as_unescaped_strings() {
    for value in [
        "0xFF", "0Xabcd", "+0xFF", "-0xFF", "0o77", "0O77", "0b11", "0B11",
    ] {
        let escaped = format!("\"{}\"", value.replacen('0', "\\x30", 1));
        let unescaped = format!("\"{value}\"");
        for template in [
            "[$value, abc]\n",
            "{$value: abc}\n",
            "- {value: $value, other: abc}\n- {value: ordinary, other: abc}\n",
        ] {
            for width in [12, 20, 40] {
                let input = template.replace("$value", &escaped);
                let expected = format_yaml(&template.replace("$value", &unescaped), width);
                let formatted = format_yaml(&input, width);
                assert_eq!(formatted, expected, "input: {input:?}, width: {width}");
                assert_eq!(
                    format_yaml(&formatted, width),
                    formatted,
                    "second pass for input: {input:?}, width: {width}",
                );
            }
        }
    }
}
