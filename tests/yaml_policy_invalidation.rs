use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions};
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
