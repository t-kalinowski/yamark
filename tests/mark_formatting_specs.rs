use std::path::Path;

use yamark::core::document::{FormatOptions, MarkdownWrap};
use yamark::workspace::format_source_for_path;

fn markdown_options(width: usize) -> FormatOptions {
    FormatOptions {
        markdown_wrap: MarkdownWrap::Column,
        markdown_wrap_at_column: width,
        ..FormatOptions::default()
    }
}

fn format_markdown(input: &str, width: usize) -> String {
    format_source_for_path(
        Path::new(r#"input.md"#),
        input.to_owned(),
        markdown_options(width),
        None,
    )
    .unwrap()
    .output
}

fn format_yaml(input: &str, options: FormatOptions) -> String {
    format_source_for_path(Path::new(r#"input.yaml"#), input.to_owned(), options, None)
        .unwrap()
        .output
}

#[test]
fn yaml_sequence_values_that_start_with_option_flags_stay_unquoted() {
    let input = r#"dependencies:
  - pip:
    - -r requirements.txt
"#;
    let expected = r#"dependencies:
  - pip:
    - -r requirements.txt
"#;

    assert_eq!(format_yaml(input, FormatOptions::default()), expected);
}

#[test]
fn yaml_dash_prefixed_strings_are_plain_when_dash_is_not_an_indicator() {
    let input = r#"options:
  command: "--no-cache"
  requirement: "-r requirements.txt"
  define: "-Dname=value"
args:
  - "--flag"
  - "-Xclang"
  - "-r requirements.txt"
flow_args: ["--flag", "-Dname=value", "-Xclang"]
"#;
    let expected = r#"options:
  command: --no-cache
  requirement: -r requirements.txt
  define: -Dname=value
args:
  - --flag
  - -Xclang
  - -r requirements.txt
flow_args: [--flag, -Dname=value, -Xclang]
"#;

    assert_eq!(format_yaml(input, FormatOptions::default()), expected);
}

#[test]
fn yaml_dash_prefixed_strings_that_change_structure_or_type_stay_quoted() {
    let input = r#"negative: "-1"
sequence_like: "- item"
dash_only: "-"
question_like: "? item"
"#;

    assert_eq!(format_yaml(input, FormatOptions::default()), input);
}

#[test]
fn quarto_chunk_options_stay_in_the_header_when_they_fit() {
    let input = r#"```{r, eval = FALSE}
my_pkg_data <- sample(1000)
usethis::use_data(my_pkg_data)
```
"#;

    assert_eq!(format_markdown(input, 72), input);
}

#[test]
fn markdown_column_wrap_does_not_start_commonmark_list_item() {
    let input = "I like to add numbers and things, like these: 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8\n";
    let expected = "I like to add numbers and things, like these: 1 + 2 + 3 + 4 + 5 + 6 +\n7 + 8\n";

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn wrapped_footnote_continuation_lines_use_two_spaces() {
    let input = r#"[^data-1]: If you don't know much about R environments and what makes them special, a great resource is the [Environments chapter](https://adv-r.hadley.nz/environments.html) of Advanced R.
"#;
    let expected = r#"[^data-1]: If you don't know much about R environments and what makes
  them special, a great resource is the
  [Environments chapter](https://adv-r.hadley.nz/environments.html) of
  Advanced R.
"#;

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn blockquote_paragraphs_wrap_with_quote_marker_width_reserved() {
    let input = r#"> One two three four five six seven eight
>
> Nine ten eleven twelve thirteen fourteen fifteen sixteen
"#;
    let expected = r#"> One two three four five six seven
> eight
>
> Nine ten eleven twelve thirteen
> fourteen fifteen sixteen
"#;

    assert_eq!(format_markdown(input, 40), expected);
}

#[test]
fn blockquote_list_items_wrap_with_quote_marker_width_reserved() {
    let input = r#"> - One two three four five six seven goes
"#;
    let expected = r#"> - One two three four five six seven
>   goes
"#;

    assert_eq!(format_markdown(input, 40), expected);
}

#[test]
fn blockquote_wrapping_between_markdown_paragraphs_preserves_blocks() {
    let input = r#"Intro paragraph stays outside the quote.

> One two three four five six seven eight

Final paragraph stays outside the quote.
"#;
    let expected = r#"Intro paragraph stays outside the quote.

> One two three four five six seven
> eight

Final paragraph stays outside the quote.
"#;

    assert_eq!(format_markdown(input, 40), expected);
}

#[test]
fn blockquote_list_wrapping_between_markdown_paragraphs_preserves_blocks() {
    let input = r#"Intro paragraph stays outside the quoted list.

> - One two three four five six seven goes

Final paragraph stays after the quoted list.
"#;
    let expected = r#"Intro paragraph stays outside the quoted
list.

> - One two three four five six seven
>   goes

Final paragraph stays after the quoted
list.
"#;

    assert_eq!(format_markdown(input, 40), expected);
}

#[test]
fn long_link_targets_split_as_part_of_surrounding_sentence_flow() {
    let input = r#"This problem is hardly unique to R.
Many applications need to leave notes to themselves.
It is best to comply with external conventions, which in this case means the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html).
You need to use the official locations for persistent file storage, because it's the responsible and courteous thing to do and also to comply with CRAN policies.
"#;
    let expected = r#"This problem is hardly unique to R. Many applications need to leave
notes to themselves. It is best to comply with external conventions,
which in this case means the [XDG Base Directory Specification](
  https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
). You need to use the official locations for persistent file storage,
because it's the responsible and courteous thing to do and also to
comply with CRAN policies.
"#;

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn long_link_targets_split_with_following_words_when_no_punctuation_follows() {
    let input = r#"For setup, read [installation guide](https://example.com/products/cloud/sdk/install/linux/enterprise) before editing the configuration file.
"#;
    let expected = r#"For setup, read [installation guide](
  https://example.com/products/cloud/sdk/install/linux/enterprise
) before editing the configuration file.
"#;

    assert_eq!(format_markdown(input, 58), expected);
}

#[test]
fn long_link_targets_split_with_following_words_when_punctuation_follows() {
    let input = r#"For setup, read [installation guide](https://example.com/products/cloud/sdk/install/linux/enterprise), then edit the configuration file.
"#;
    let expected = r#"For setup, read [installation guide](
  https://example.com/products/cloud/sdk/install/linux/enterprise
), then edit the configuration file.
"#;

    assert_eq!(format_markdown(input, 58), expected);
}

#[test]
fn nested_image_links_keep_a_short_outer_destination_inline() {
    let input = r#"[![Microsoft Foundry Discord](https://dcbadge.limes.pink/api/server/nTYy5BXMWG)](https://discord.gg/nTYy5BXMWG)
"#;
    let expected = r#"[![Microsoft Foundry Discord](
  https://dcbadge.limes.pink/api/server/nTYy5BXMWG
)](https://discord.gg/nTYy5BXMWG)
"#;

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn adjacent_short_links_are_not_eagerly_split() {
    let input = r#"If the data is potentially sensitive, such as user credentials, it is recommended to obtain the user's consent to store it, i.e. to require interactive consent when initiating the cache. Also consider that the user's operating system or command line tools might provide a means of secure storage that is superior to any DIY solution that you might implement. The packages [keyring](https://cran.r-project.org/package=keyring), [gitcreds](https://gitcreds.r-lib.org), and [credentials](https://docs.ropensci.org/credentials/) are examples of packages that tap into externally-provided tooling. Before embarking on any creative solution for storing secrets, consider that your effort is probably better spent integrating with an established tool.
"#;
    let expected = r#"If the data is potentially sensitive, such as user credentials, it is
recommended to obtain the user's consent to store it, i.e. to require
interactive consent when initiating the cache. Also consider that the
user's operating system or command line tools might provide a means of
secure storage that is superior to any DIY solution that you might
implement. The packages
[keyring](https://cran.r-project.org/package=keyring),
[gitcreds](https://gitcreds.r-lib.org), and
[credentials](https://docs.ropensci.org/credentials/) are examples of
packages that tap into externally-provided tooling. Before embarking on
any creative solution for storing secrets, consider that your effort is
probably better spent integrating with an established tool.
"#;

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn markdown_wrapping_is_idempotent_after_a_short_link_moves_to_its_own_line() {
    let input = r#"This post provides an overview of these capabilities in Quarto. For more detail about all the features Quarto for authoring tables, see [Tables](/docs/authoring/tables.qmd).
"#;
    let once = format_markdown(input, 72);
    let twice = format_markdown(&once, 72);

    assert_eq!(twice, once);
}

#[test]
fn markdown_wrapping_preserves_mixed_inline_spans() {
    let input = r#"A paragraph with `inline code spans`, [short links](https://example.com/short), <https://example.com/autolink>, $x + y$, and {#attrs} keeps every protected span while wrapping the surrounding prose.
"#;
    let expected = r#"A paragraph with `inline code spans`,
[short links](https://example.com/short),
<https://example.com/autolink>, $x + y$, and {#attrs} keeps every
protected span while wrapping the surrounding prose.
"#;

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn markdown_paragraph_spacing_normalizes_blank_line_runs() {
    let input = r#"First paragraph.


Second paragraph.



Third paragraph.
"#;
    let expected = r#"First paragraph.

Second paragraph.

Third paragraph.
"#;

    assert_eq!(format_markdown(input, 72), expected);
}

#[test]
fn markdown_nested_emphasis_marker_runs_preserve_source_spelling() {
    let input = r#"This keeps ***asterisk triple***, ___underscore triple___, **_strong em_**, __*strong em*__, *__em strong__*, and _**em strong**_ exactly.
"#;

    assert_eq!(format_markdown(input, 200), input);
}

#[test]
fn markdown_spaced_marker_text_and_thematic_breaks_do_not_mix() {
    let input = r#"A paragraph mentions literal markers: * and * * and * * * before continuing.

***

A paragraph mentions literal dashes: --- before continuing.

---
"#;
    let expected = r#"A paragraph mentions literal markers: * and * * and * * * before continuing.

---

A paragraph mentions literal dashes: --- before continuing.

---
"#;

    assert_eq!(format_markdown(input, 200), expected);
}
