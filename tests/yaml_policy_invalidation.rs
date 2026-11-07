use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions, MarkdownWrap};
use yamark::core::parser::format_source;
use yamark::plugins::PluginRegistry;

#[test]
fn template_file_policy_is_position_independent_across_widths_and_line_endings() {
    let config = Config::default();
    let plugins = PluginRegistry::default();
    for newline in ["\n", "\r\n", "\r"] {
        let directive = format!("# fmt: template.delimiters \"<<\" \">>\" scope=file{newline}");
        let body = "held: &saved # parent\n  'Café value'\nquoted: \"<< Café >>\"\nitems: [<< a,b >>]\nok: [x,y]\n"
            .replace('\n', newline);
        for line_width in [12, 80] {
            let options = FormatOptions {
                line_width,
                ..FormatOptions::default()
            };
            let format =
                |input| format_source(FileKind::Yaml, input, options, &config, &plugins).unwrap();
            let early = format(format!("{directive}{body}"));
            let late = format(format!("{body}{directive}"));
            assert_eq!(
                early.strip_prefix(&directive).unwrap(),
                late.strip_suffix(&directive).unwrap(),
                "line width {line_width}, line ending {newline:?}"
            );
            assert!(late.contains(&format!("ok: [x, y]{newline}")));
            assert_eq!(format(late.clone()), late);
        }
    }
}

fn format_document(kind: FileKind, input: &str, options: FormatOptions) -> String {
    format_source(
        kind,
        input.to_owned(),
        options,
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap()
}

#[test]
fn layout_only_file_policies_refresh_plans_without_losing_source_boundaries() {
    for newline in ["\n", "\r\n", "\r"] {
        let body = "held: &saved # parent\n  'Café value'\ntemplated: [\"{{ keep   bytes }}\",  1]\nitems:\n  - one\n  - two\n"
            .replace('\n', newline);
        for compact in [false, true] {
            // Both directions must replace an existing emission plan, even
            // though preservation decisions remain valid for this delta.
            let options = FormatOptions {
                yaml_compact: !compact,
                ..FormatOptions::default()
            };
            let directive = format!("# fmt: compact={compact} scope=file{newline}");
            let early = format_document(FileKind::Yaml, &format!("{directive}{body}"), options);
            let late = format_document(FileKind::Yaml, &format!("{body}{directive}"), options);
            assert_eq!(
                early.strip_prefix(&directive).unwrap(),
                late.strip_suffix(&directive).unwrap(),
                "compact {compact}, line ending {newline:?}"
            );
            assert!(late.contains(&format!(
                "held: &saved # parent{newline}  'Café value'{newline}"
            )));
            assert!(late.contains(&format!(
                "templated: [\"{{{{ keep   bytes }}}}\",  1]{newline}"
            )));
            let items = if compact {
                format!("items: [one, two]{newline}")
            } else {
                format!("items:{newline}  - one{newline}  - two{newline}")
            };
            assert!(late.contains(&items), "{late}");
            assert_eq!(format_document(FileKind::Yaml, &late, options), late);
        }
    }
}

#[test]
fn layout_only_markdown_policy_refreshes_already_planned_nested_yaml() {
    let initial = FormatOptions {
        markdown_wrap: MarkdownWrap::None,
        ..FormatOptions::default()
    };
    let effective = FormatOptions {
        markdown_wrap: MarkdownWrap::Paragraph,
        markdown_canonical: true,
        ..initial
    };
    for newline in ["\n", "\r\n", "\r"] {
        let yaml = "held: &saved # parent\n  'Café value'\ntemplated: [\"{{ keep   bytes }}\",  1]\nbody: !markdown |\n  Keep __strong__.\n  Continue on another line.\nattached: !markdown\n  |\n    Keep __attached__.\nok: [x,y]\n"
            .replace('\n', newline);
        let directive = format!("<!-- fmt: canonical=true wrap=paragraph scope=file -->{newline}");
        let input = format!("```yaml{newline}{yaml}```{newline}{newline}{directive}");
        let output = format_document(FileKind::Markdown, &input, initial);
        // A late outer policy reaches an already-planned YAML arena. Compare
        // that replan with fresh YAML formatting under the effective options.
        let expected_yaml = format_document(FileKind::Yaml, &yaml, effective);
        assert!(expected_yaml.contains(&format!(
            "body: !markdown |{newline}  Keep **strong**. Continue on another line.{newline}"
        )));
        assert!(expected_yaml.contains(&format!(
            "attached: !markdown |{newline}  Keep **attached**.{newline}"
        )));
        assert_eq!(
            output,
            format!("```yaml{newline}{expected_yaml}```{newline}{newline}{directive}")
        );
        assert_eq!(
            format_document(FileKind::Markdown, &output, initial),
            output
        );
    }
}

#[test]
fn mixed_layout_and_template_file_policies_invalidate_preservation_in_either_order() {
    for newline in ["\n", "\r\n", "\r"] {
        let template = format!("# fmt: template.delimiters \"<<\" \">>\" scope=file{newline}");
        let compact = format!("# fmt: compact scope=file{newline}");
        let body = "held: &saved # parent\n  'Café value'\nquoted: \"<< Café >>\"\nitems: [<< a,b >>]\nedit:\n  - a\n  - b\nok: [x,y]\n"
            .replace('\n', newline);
        for directives in [
            format!("{template}{compact}"),
            format!("{compact}{template}"),
        ] {
            for line_width in [12, 80] {
                let options = FormatOptions {
                    line_width,
                    ..FormatOptions::default()
                };
                let early =
                    format_document(FileKind::Yaml, &format!("{directives}{body}"), options);
                let late = format_document(FileKind::Yaml, &format!("{body}{directives}"), options);
                assert_eq!(
                    early.strip_prefix(&directives).unwrap(),
                    late.strip_suffix(&directives).unwrap(),
                    "line width {line_width}, line ending {newline:?}, directives {directives:?}"
                );
                assert!(late.contains(&format!("items: [<< a,b >>]{newline}")));
                assert!(late.contains(&format!("edit: [a, b]{newline}")));
                assert!(late.contains(&format!("ok: [x, y]{newline}")));
                assert_eq!(format_document(FileKind::Yaml, &late, options), late);
            }
        }
    }
}

#[test]
fn empty_file_delta_keeps_nested_yaml_effective_options() {
    let options = FormatOptions {
        yaml_compact: true,
        markdown_wrap: MarkdownWrap::Paragraph,
        markdown_canonical: true,
        ..FormatOptions::default()
    };
    for newline in ["\n", "\r\n", "\r"] {
        let yaml = "held: &saved # parent\n  'Café value'\ntemplated: [\"{{ keep   bytes }}\",  1]\nitems:\n  - one\n  - two\nbody: !markdown |\n  Keep __strong__.\n  Continue on another line.\nok: [x,y]\n"
            .replace('\n', newline);
        let directive = format!("<!-- fmt: scope=file -->{newline}");
        let input = format!("```yaml{newline}{yaml}```{newline}{newline}{directive}");
        let output = format_document(FileKind::Markdown, &input, options);
        // This directive has an empty delta, but still exercises nested
        // replanning with nondefault effective options and retained source.
        let expected_yaml = format_document(FileKind::Yaml, &yaml, options);
        assert!(expected_yaml.contains(&format!("items: [one, two]{newline}")));
        assert!(expected_yaml.contains(&format!(
            "body: !markdown |{newline}  Keep **strong**. Continue on another line.{newline}"
        )));
        assert_eq!(
            output,
            format!("```yaml{newline}{expected_yaml}```{newline}{newline}{directive}")
        );
        assert_eq!(
            format_document(FileKind::Markdown, &output, options),
            output
        );
    }
}
