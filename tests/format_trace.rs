use assert_cmd::Command;
use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions};
use yamark::core::parser::{format_source_report, format_source_report_with_trace};
use yamark::plugins::PluginRegistry;

#[test]
fn yaml_trace_collection_is_opt_in_for_formatting() {
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let input = "name: value\n".to_owned();

    let formatted = format_source_report(
        FileKind::Yaml,
        input.clone(),
        FormatOptions::default(),
        &config,
        &plugins,
    )
    .unwrap();
    assert!(formatted.trace.is_none());

    let traced = format_source_report_with_trace(
        FileKind::Yaml,
        input,
        FormatOptions::default(),
        &config,
        &plugins,
    )
    .unwrap();
    let trace = traced.trace.unwrap();
    assert_eq!(trace.planned_rendered_scalars, 2);
}

#[cfg(feature = "format-trace")]
#[test]
fn markdown_trace_reports_format_and_skip_decisions_when_feature_is_enabled() {
    let input = "\
#  Heading

<div>
raw
</div>

This paragraph is intentionally long enough to be considered for wrapping by the markdown formatter.
";
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "input.md:1:1: note: markdown trace: formatted kind=Heading emit=MarkdownHeading"
        ),
        "{stderr}"
    );
    assert!(
        stderr.contains("input.md:3:1: note: markdown trace: skipped kind=Raw emit=Copy"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "input.md:7:1: note: markdown trace: formatted kind=Paragraph emit=MarkdownParagraph"
        ),
        "{stderr}"
    );
}

#[cfg(feature = "format-trace")]
#[test]
fn markdown_trace_marks_supported_rich_inline_table_and_raw_fence_features_formatted() {
    let input = "\
Press <kbd>Cmd</kbd> + <kbd>K</kbd> and use ~~old option~~ only for history.

Read the [long guide](
  https://example.com/guide
) before continuing with the tutorial.

| Include | Target |
|---|---|
| footer | {{< include _footer.md >}} |

``` {=typst}
#pagebreak()
```

1. Build a table:

   | a | b |
   |---|---|
   | 1 | 2 |

> | Term | Meaning |
> |---|---|
> | ML | Machine learning |

::: {.callout-note}
This paragraph is intentionally long enough to be considered for wrapping inside a supported div.
:::

{{< meta title >}}

$$
x = y
$$

[^note]: See [`library()` vs `require()`](https://yihui.org/en/2014/07/library-vs-require/) and `$x + y$`.

[^long]: First paragraph of the footnote.

    Second paragraph with [**strong label**](https://example.com).
";
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "input.md:1:1: note: markdown trace: formatted kind=Paragraph emit=MarkdownParagraph"
        ),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=GfmPipeTable emit=MarkdownTable"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=CodeFence emit=MarkdownCodeFence"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=List emit=MarkdownList"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=Blockquote emit=MarkdownBlockquote"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=QuartoDiv emit=MarkdownDiv"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=Shortcode emit=MarkdownOpaque"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=DisplayMath emit=MarkdownOpaque"),
        "{stderr}"
    );
    assert!(
        stderr.contains("markdown trace: formatted kind=FootnoteDefinition emit=MarkdownParagraph"),
        "{stderr}"
    );
    assert!(!stderr.contains("markdown trace: skipped"), "{stderr}");
}

#[cfg(feature = "format-trace")]
#[test]
fn markdown_trace_marks_common_opaque_code_fences_formatted() {
    let input = "\
```
literal output
  preserve spacing
```

```rust
fn main() {
    println!(\"hello\");
}
```

```console
$ yamark format .
```

```mermaid
flowchart LR
  A --> B
```
";
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, input);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        stderr
            .matches("markdown trace: formatted kind=CodeFence emit=MarkdownCodeFence")
            .count(),
        4,
        "{stderr}"
    );
    assert!(!stderr.contains("markdown trace: skipped"), "{stderr}");
}

#[cfg(feature = "format-trace")]
#[test]
fn markdown_trace_marks_reference_definitions_formatted() {
    let input = "\
Read the [guide] before opening the [issue list][issues].

[guide]: https://example.com/guide
[issues]: https://example.com/issues
";
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, input);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "input.md:1:1: note: markdown trace: formatted kind=Paragraph emit=MarkdownParagraph"
        ),
        "{stderr}"
    );
    assert_eq!(
        stderr
            .matches("markdown trace: formatted kind=ReferenceDefinition emit=MarkdownOpaque")
            .count(),
        2,
        "{stderr}"
    );
    assert!(!stderr.contains("markdown trace: skipped"), "{stderr}");
}

#[cfg(feature = "format-trace")]
#[test]
fn markdown_trace_marks_alpha_and_strict_nested_ordered_lists_formatted() {
    let input = "\
  a. License grant.
  b. Other rights.

1. Construct an image `X` with diagonal edges.
    1. What happens if you apply the kernel `K` to the image?
    1. What happens if you transpose `X` before applying it?
1. Design some kernels manually.
";
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        stderr
            .matches("markdown trace: formatted kind=List emit=MarkdownList")
            .count(),
        2,
        "{stderr}"
    );
    assert!(!stderr.contains("markdown trace: skipped"), "{stderr}");
}

#[cfg(not(feature = "format-trace"))]
#[test]
fn markdown_trace_is_not_present_without_format_trace_feature() {
    let input = "# Heading\n\nparagraph\n";
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--diagnostics", "--stdin-file-path", "input.md"])
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("markdown trace:"), "{stderr}");
}
