use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions};
use yamark::core::parser::format_source;
use yamark::plugins::PluginRegistry;

fn format(kind: FileKind, source: &str, options: FormatOptions) -> String {
    format_source(
        kind,
        source.to_owned(),
        options,
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap()
}

#[test]
fn yaml_fences_format_the_cleaned_body_once_and_keep_preserved_boundaries() {
    let body = "abstract: > \n  The characteristic theme of the works of Stone is \n  the bridge between culture and society. ...\n";
    let cleaned = body
        .lines()
        .map(|line| format!("{}\n", line.trim_end_matches([' ', '\t'])))
        .collect::<String>();
    for newline in ["\n", "\r\n", "\r"] {
        let body = body.replace('\n', newline);
        let cleaned = cleaned.replace('\n', newline);
        for line_width in [24, 72] {
            let options = FormatOptions {
                line_width,
                ..FormatOptions::default()
            };
            for language in ["yaml", "yml"] {
                let fence = format!("```{language}{newline}{body}```{newline}");
                let expected_body = format(FileKind::Yaml, &cleaned, options);
                let expected = format!("```{language}{newline}{expected_body}```{newline}");
                let first = format(FileKind::Markdown, &fence, options);
                assert_eq!(first, expected);
                assert_eq!(format(FileKind::Markdown, &first, options), first);
                for preserved in [
                    format!("<!-- fmt: skip -->{newline}{fence}"),
                    format!("<!-- fmt: off -->{newline}{fence}<!-- fmt: on -->{newline}"),
                    format!("{fence}{newline}<!-- fmt: skip file -->{newline}"),
                ] {
                    assert_eq!(format(FileKind::Markdown, &preserved, options), preserved);
                }
            }
            let standalone = format(FileKind::Yaml, &body, options);
            assert!(standalone.contains(&format!("Stone is {newline}")));
            assert_eq!(format(FileKind::Yaml, &standalone, options), standalone);
        }
    }
}

#[test]
fn late_markdown_policy_replans_the_cleaned_yaml_source() {
    let body = "body: !markdown |\n  Keep __strong__. \nitems: [a,b] \n";
    for newline in ["\n", "\r\n", "\r"] {
        let body = body.replace('\n', newline);
        let options = FormatOptions::default();
        let directive = format!("<!-- fmt: canonical=true scope=file -->{newline}");
        let fence = format!("```yaml{newline}{body}```{newline}");
        let early = format(FileKind::Markdown, &format!("{directive}{fence}"), options);
        let late = format(FileKind::Markdown, &format!("{fence}{directive}"), options);
        assert_eq!(
            early.strip_prefix(&directive),
            late.strip_suffix(&directive)
        );
        assert!(late.contains("Keep **strong**."));
        assert_eq!(format(FileKind::Markdown, &late, options), late);
    }
}
