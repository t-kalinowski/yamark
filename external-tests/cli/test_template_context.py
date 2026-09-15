"""Template preservation follows Markdown syntax and decoded YAML contents."""

from __future__ import annotations

import pytest
from _support import run_cli_case


@pytest.mark.parametrize(
    "html",
    [
        '<span title="> {{ foo }} _keep_"/>',
        '<span title="> {{ foo }}">keep   _this_</span>',
        '<span title="> </span> {{ foo }} _keep_">keep   _this_</span>',
        '<span>{{ "</span>" ~ "keep   _this_" }}</span>',
        '<span>{{ "<span>" ~ "keep   _this_" }}</span>',
        '<span>{ {{ "{" ~ "</span>" ~ "keep   _this_" }}</span>',
        "<span><span>x</span>keep   _this_ {{ foo }}</span>",
        "<span><SPAN><span>x</span></SPAN>keep   _this_ {{ foo }}</sPaN>",
        '<span><span title="> </span>">x</span>keep   _this_ {{ foo }}</span>',
        '<span><b title="<span> </span>">x</b><!-- </span> -->keep   _this_ {{ foo }}</span>',
        "<span><span/><span>x</span>keep   _this_ {{ foo }}</span>",
        r"<span>keep   _this_ {{ foo }}\</span>",
        *[
            f'<span><{tag}>const x="<span>"</{tag}>keep   _this_ {{{{ foo }}}}</span>'
            for tag in [
                "script",
                "style",
                "textarea",
                "title",
                "iframe",
                "xmp",
                "noembed",
                "noframes",
            ]
        ],
    ],
)
@pytest.mark.parametrize("wrap", ["sentence", "20"])
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_html_regions_stay_opaque_while_prose_wraps(
    html: str, wrap: str, canonical: str
) -> None:
    source = f"Before\n{html}\nafter.\n"
    expected = f"Before {html} after.\n" if wrap == "sentence" else source
    for text in [source, expected]:
        run_cli_case(
            f"yamark format {canonical} --wrap {wrap} --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("tag", ["script", "style", "textarea", "title"])
def test_raw_text_html_does_not_nest_inside_a_heading(tag: str) -> None:
    html = f'<{tag}>const x="<{tag}>"; keep   _this_ {{{{ foo }}}}</{tag.upper()}>'
    source = f"#   Heading {html}   ##\n"
    expected = f"# Heading {html}\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --canonical --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("markers", [("<", ">"), ("%", "%")])
@pytest.mark.parametrize("quote", ['"', "'", "`"])
@pytest.mark.parametrize("wrap", ["sentence", "20"])
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_inline_shortcodes_keep_quoted_delimiters(
    markers: tuple[str, str], quote: str, wrap: str, canonical: str
) -> None:
    opening, closing = markers
    shortcode = (
        f"{{{{{opening} note text={quote}}}}} keep   _this_{quote} {closing}}}}}"
    )
    source = f"Before   {shortcode} after.\n"
    expected = (
        f"Before {shortcode} after.\n"
        if wrap == "sentence"
        else f"Before\n{shortcode}\nafter.\n"
    )
    for text in [source, expected]:
        run_cli_case(
            f"yamark format {canonical} --wrap {wrap} --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("indent", [4, 6])
@pytest.mark.parametrize("prefix", ["", "> "])
def test_code_spans_do_not_cross_deeply_indented_list_children(
    indent: int, prefix: str
) -> None:
    paragraph = (
        f"{prefix}- Before `<% keep   this %>\n{prefix}{' ' * indent}- child `\n"
    )
    source = paragraph + "\nFollowing\nprose.\n"
    expected = paragraph + "\nFollowing prose.\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    "content",
    [
        "`{{ keep  \nthis }}`",
        "`{{ keep\\\nthis }}`",
        "`before  \n{{ foo }}`",
        "`{{ foo }}\nafter  \ncode`",
        "Before {{ foo }}  \nafter.",
    ],
)
@pytest.mark.parametrize("prefix", ["", "> ", "- "])
def test_template_blocks_with_apparent_hard_breaks_are_preserved(
    content: str, prefix: str
) -> None:
    continuation = "  " if prefix == "- " else prefix
    paragraph = prefix + ("\n" + continuation).join(content.splitlines()) + "\n"
    source = paragraph + "\nFollowing\nprose.\n"
    expected = paragraph + "\nFollowing prose.\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    "template",
    [
        "`<% result = `printf 'keep   this'` %>`",
        "`{{ printf `keep   this` }}`",
        "`{{ first }} {{ printf `keep   this` }}`",
        '`{{< note text="}}` keep   _this_" >}}',
        "`{{% note text='}}` keep   _this_' %}}",
        '`{{ "}}` keep   _this_" }}',
        '`{% set text = "%}` keep   _this_" %}',
    ],
)
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_template_pairs_crossing_code_spans_preserve_the_block(
    template: str, canonical: str
) -> None:
    paragraph = f"Before\n{template}\nafter.\n"
    source = paragraph + "\nFollowing\nprose.\n"
    expected = paragraph + "\nFollowing prose.\n"
    for text in [source, expected]:
        run_cli_case(
            f"yamark format {canonical} --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    ("suffix", "source"),
    [
        ("md", 'First {{ printf "keep   this"\n}} after.\n'),
        ("md", '> First {{ printf "keep   this"\n> }} after.\n'),
        ("yaml", 'doc: !markdown |\n  First {{ printf "keep   this"\n  }} after.\n'),
    ],
)
def test_braced_templates_crossing_source_lines_are_preserved(
    suffix: str, source: str
) -> None:
    run_cli_case(
        f"yamark format --wrap sentence --stdin-file-path input.{suffix} --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("value", ["{{ 'keep this' }}", "one {{ 'keep this' }} two"])
def test_braced_fig_alt_values_are_not_split(value: str) -> None:
    source = f'![x](url){{fig-alt="{value}"}}\n'
    expected = f'![x](url){{\n  fig-alt="{value}"\n}}\n'
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap 20 --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    "expression", ['{{ "keep   this" }}', '{% set x = "keep   this" %}']
)
@pytest.mark.parametrize("suffix", ["", "suffix"])
@pytest.mark.parametrize("wrap", ["sentence", "20"])
def test_attached_templates_stay_in_one_token(
    expression: str, suffix: str, wrap: str
) -> None:
    token = "prefix" + expression + suffix
    source = f"First\nsentence with {token}. Second\nsentence.\n"
    expected = (
        f"First sentence with {token}.\nSecond sentence.\n"
        if wrap == "sentence"
        else f"First sentence with\n{token}.\nSecond sentence.\n"
    )
    for text in [source, expected]:
        run_cli_case(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("prefix", ["", "λ "])
@pytest.mark.parametrize(
    "link",
    ["[[x](foo](outer))", "[$x](outer)$", '[<span title="](outer)">]'],
)
def test_template_detection_respects_link_label_boundaries(
    prefix: str, link: str
) -> None:
    source = prefix + link + " <% value %>\n"
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize(
    "argument", ["don't   alter", 'a "   quote', "a `   tick", "nested {don't   alter}"]
)
@pytest.mark.parametrize("prefix", ["", "\\mycommand"])
def test_generic_brace_groups_keep_literal_quotes(argument: str, prefix: str) -> None:
    command = prefix + "{" + argument + "}"
    source = f"#   H {command}   ##\n"
    expected = f"# H {command}\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    ("suffix", "source"),
    [
        ("yaml", 'doc: !markdown |\n  {{ printf "keep   this"\n\n  }}\n'),
        ("md", '> {{ printf "keep   this"\n>\n> }}\n'),
        ("md", '- {{ printf "keep   this"\n\n  }}\n'),
        ("md", '> {{ printf "keep   this"\n> # Heading\n> }}\n'),
        ("md", '> Intro.\n>\n> > {{ printf "keep   this"\n> outer }}\n'),
        ("md", '> Intro.\n>\n> > > {{ printf "keep   this"\n> outer }}\n'),
    ],
)
def test_balanced_templates_crossing_child_blocks_are_preserved(
    suffix: str, source: str
) -> None:
    run_cli_case(
        f"yamark format --wrap sentence --stdin-file-path input.{suffix} --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("tag", ["raw", "verbatim", "verbatim example"])
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_template_tag_names_do_not_change_markdown_formatting(
    tag: str, canonical: str
) -> None:
    opening, closing = f"{{% {tag} %}}", f"{{% end{tag} %}}"
    source = f"Before\n{opening}keep   this{closing}\nafter.\n"
    expected = f"Before {opening}keep this{closing} after.\n"
    for text in [source, expected]:
        run_cli_case(
            f"yamark format {canonical} --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("tag", ["raw", "verbatim", "verbatim example"])
def test_template_tags_inside_inline_code_allow_wrapping(tag: str) -> None:
    code = f"`{{% {tag} %}}keep   this{{% end{tag} %}}`"
    source = f"Before\n{code}\nafter.\n"
    expected = f"Before {code} after.\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    "raw_html",
    [
        "<!-- `{% raw %}` keep   _this_ `{% endraw %}` -->",
        "<!-- {% raw %} keep   _this_ {% endraw %} -->",
        r"<!-- \`{% raw %}\` keep   _this_ \`{% endraw %}\` -->",
        "<?check `{% raw %}` keep   _this_ `{% endraw %}` ?>",
        "<![CDATA[`{% raw %}` keep   _this_ `{% endraw %}`]]>",
        "<!YAMARK `{% raw %}` keep   _this_ `{% endraw %}`>",
    ],
)
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_templates_in_raw_inline_html_preserve_the_block(
    raw_html: str, canonical: str
) -> None:
    heading = f"#   H {raw_html}   ##\n"
    source = heading + "\nFollowing\nprose.\n"
    expected = heading + "\nFollowing prose.\n"
    for text in [source, expected]:
        run_cli_case(
            f"yamark format {canonical} --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


def test_raw_html_spelling_inside_inline_code_still_allows_wrapping() -> None:
    code = "`<!-- <% keep   this %> -->`"
    source = f"First\nsentence with {code}. Second\nsentence.\n"
    expected = f"First sentence with {code}.\nSecond sentence.\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("quote", ['"', "'"])
@pytest.mark.parametrize("heading", [False, True])
def test_template_in_quoted_html_attribute_is_not_code(
    quote: str, heading: bool
) -> None:
    tag = f"<span title={quote}> `<% keep   this %>`{quote}>text</span>"
    source = (
        f"#   Heading {tag}   ##\n"
        if heading
        else f"First\nsentence with {tag}. Second\nsentence.\n"
    )
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("destination", [r"foo\_bar", r"foo\(bar\)"])
def test_template_in_escaped_link_destination_is_not_code(destination: str) -> None:
    source = f"#   Heading [x]({destination}/`<%value%>`)   ##\n"
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("spacing", ["", " ", "\t"])
def test_template_in_image_attributes_is_not_code(spacing: str) -> None:
    source = f'![x](url){spacing}{{fig-alt="`<% keep   this %>`"}}\n'
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("template", ["{{ keep   this }}", "`{{ keep   this }}`"])
def test_braced_templates_survive_image_attribute_normalization(template: str) -> None:
    source = f'![x](url) {{ fig-alt="{template}" }}\n'
    expected = f'![x](url){{fig-alt="{template}"}}\n'
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("heading", ["# Title", "Title\n====="])
@pytest.mark.parametrize("prefix", ["", "> "])
def test_template_in_heading_attributes_is_not_code(heading: str, prefix: str) -> None:
    lines = heading.splitlines()
    lines[0] += ' {fig-alt="`<%= %q(keep   this) %>`"}'
    source = "".join(prefix + line + "\n" for line in lines)
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize(
    "expression",
    [
        '{{ "}}" ~ "keep   this" }}',
        "{{ '}}' ~ 'keep   this' }}",
        r'{{ "escaped \"}}" ~ "keep   this" }}',
        '{{ render({"label": "} keep   this", "other": "{{"}) }}',
    ],
)
@pytest.mark.parametrize("wrap", ["sentence", "20"])
def test_quoted_braces_stay_inside_the_template(expression: str, wrap: str) -> None:
    source = f"First\nsentence with {expression}. Second\nsentence.\n"
    expected = (
        f"First sentence with {expression}.\nSecond sentence.\n"
        if wrap == "sentence"
        else f"First sentence with\n{expression}.\nSecond sentence.\n"
    )
    for text in [source, expected]:
        run_cli_case(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("backslash", [r"\\", r"\u005c", r"\x5c"])
def test_quoted_markdown_yaml_uses_decoded_backtick_escapes(backslash: str) -> None:
    scalar = (
        f'doc: !markdown "First   sentence with {backslash}`<% keep   this %>'
        f'{backslash}`. Second sentence.\\n"\n'
    )
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.yaml --verify",
        stdin=scalar + "items: [a,b]\n",
        stdout=scalar + "items: [a, b]\n",
    )


@pytest.mark.parametrize(
    "source",
    [
        "-   Before `\n-   <% keep   this %>\n-   After `\n",
        (
            "First `\n:   First description.\nMiddle\n:   <% keep   this %>\n"
            "Last `\n:   Last description.\n"
        ),
        "> #   First `\n>\n> <% keep   this %>\n>\n> #   Last `\n",
        "> > #   First `\n> >\n> > <% keep   this %>\n> >\n> > #   Last `\n",
    ],
    ids=["list-items", "definitions", "blockquote-paragraphs", "nested-blockquote"],
)
def test_code_spans_do_not_cross_container_blocks(source: str) -> None:
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("prefix", ["- ", "> "])
def test_multiline_template_code_keeps_its_container_paragraph(prefix: str) -> None:
    continuation = "  " if prefix == "- " else prefix
    source = (
        f"{prefix}First\n{continuation}sentence with `<% keep\n"
        f"{continuation}this %>`. Second\n{continuation}sentence.\n"
    )
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )
