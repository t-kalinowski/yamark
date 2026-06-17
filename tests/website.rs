use std::fs;
use std::path::Path;

#[test]
fn public_materials_do_not_refer_to_legacy_product_names() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let forbidden = [
        concat!("yamark", "2"),
        concat!("yamark", "-proto"),
        concat!("yamark", "-ext"),
        concat!("yamark", "_ext"),
        concat!("yamark", "-next"),
        concat!("yamark", "_next"),
        concat!("previous", " product"),
        concat!("current", " product ", "under", " audit"),
        concat!("earlier", " iteration"),
    ];
    let mut files = vec![
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("pyproject.toml"),
        root.join("uv.lock"),
        root.join("README.md"),
    ];
    collect_text_files(&root.join("docs"), &mut files);
    collect_text_files(&root.join("website"), &mut files);
    collect_text_files(&root.join("editors"), &mut files);
    collect_text_files(&root.join("tools"), &mut files);

    for file in files {
        let contents = fs::read_to_string(&file).unwrap();
        for term in forbidden {
            assert!(
                !contents.contains(term),
                "{} should not refer to legacy product name or lineage term {term:?}",
                file.display()
            );
        }
    }
}

fn collect_text_files(path: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if name.starts_with('.') || name == "_site" {
                continue;
            }
            collect_text_files(&path, files);
            continue;
        }
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(
            extension,
            "R" | "json" | "js" | "md" | "py" | "qmd" | "sh" | "toml" | "yml"
        ) {
            files.push(path);
        }
    }
}

#[test]
fn public_repo_metadata_is_ready() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let license = fs::read_to_string(root.join("LICENSE")).unwrap();
    let cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let pyproject = fs::read_to_string(root.join("pyproject.toml")).unwrap();
    let readme = fs::read_to_string(root.join("README.md")).unwrap();

    assert!(license.contains("MIT License"));
    assert!(license.contains("Copyright (c) 2026 Tomasz Kalinowski"));

    for field in [
        r#"description = "Format YAML and Markdown with yamark.""#,
        r#"repository = "https://github.com/t-kalinowski/yamark""#,
        r#"homepage = "https://t-kalinowski.github.io/yamark/""#,
        r#"readme = "README.md""#,
    ] {
        assert!(cargo.contains(field), "Cargo.toml should include {field}");
    }
    assert!(cargo.contains(r#"license = "MIT""#));

    assert!(
        !pyproject.contains("Add your description here"),
        "pyproject description should not be a template placeholder"
    );
    assert!(pyproject.contains("description = \"Format YAML and Markdown with yamark.\""));
    assert!(pyproject.contains("[build-system]"));
    assert!(pyproject.contains(r#"build-backend = "maturin""#));
    assert!(pyproject.contains("[tool.maturin]"));
    assert!(pyproject.contains(r#"bindings = "bin""#));
    assert!(
        !pyproject.contains("\"pytest"),
        "pytest should not be a runtime project dependency"
    );
    assert!(
        !pyproject.contains("package = false"),
        "pyproject should remain packageable for PyPI"
    );

    assert!(readme.starts_with("# Yamark\n"));
    assert!(!readme.contains("Yamark Next"));
    for section in ["## Install", "## Usage", "## Development"] {
        assert!(
            readme.contains(section),
            "README should document public-facing {section}"
        );
    }
    for command in [
        "uvx maturin build --release",
        "cargo build --bin yamark",
        "cargo install --path .",
        "yamark format",
        "cargo test",
        "uv run external-tests/run.py",
        "npm test",
    ] {
        assert!(
            readme.contains(command),
            "README should document command {command}"
        );
    }
}

#[test]
fn ci_runs_public_readiness_checks() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();

    assert!(ci.contains("permissions:"));
    assert!(ci.contains("contents: read"));
    assert!(ci.contains("concurrency:"));
    assert!(ci.contains("cancel-in-progress: true"));

    for command in [
        "cargo fmt --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test",
        "uv run external-tests/run.py --serial",
        "npm test",
    ] {
        assert!(
            ci.contains(command),
            "CI should run public readiness command {command}"
        );
    }
}

#[test]
fn website_social_images_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let mut expected = Vec::new();
    for line in config.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("image: ") {
            expected.push(root.join(path));
        }
    }

    assert!(
        !expected.is_empty(),
        "website should configure social images"
    );
    for path in expected {
        assert!(path.is_file(), "{} should exist", path.display());
    }
}

#[test]
fn website_includes_benchmarks_page() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let benchmarks = fs::read_to_string(root.join("benchmarks.qmd")).unwrap();
    let data = fs::read_to_string(root.join("_benchmark-data.R")).unwrap();
    let rendered = fs::read_to_string(root.join("benchmarks.html.md")).unwrap();
    let rendered_index = fs::read_to_string(root.join("index.html.md")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("benchmarks.qmd"));
    assert!(benchmarks.contains("title: Benchmarks"));
    assert!(benchmarks.contains("label: benchmark-data"));
    assert!(benchmarks.contains("How to read these results"));
    assert!(benchmarks.contains("Reproducing"));

    // One comparison per input kind, each with its own native-CLI roster.
    assert!(benchmarks.contains("## Markdown"));
    assert!(benchmarks.contains("## YAML"));
    assert!(benchmarks.contains("## Markdown + front matter"));
    assert!(benchmarks.contains("## Directory"));
    assert!(benchmarks.contains("tools/bench/big.py"));
    assert!(benchmarks.contains("--files 500 --items 540"));
    assert!(benchmarks.contains("MacBook Pro"));

    // The page must say, in visible prose, why the lint fixers are not part
    // of the comparison - not bury them in a comment.
    assert!(benchmarks.contains("pymarkdown"));
    assert!(benchmarks.contains("markdownlint-cli2"));
    assert!(benchmarks.contains("not formatters"));

    // Cache handling is a disclosed part of the methodology.
    assert!(benchmarks.contains("cache"));
    assert!(benchmarks.contains("formats from scratch"));

    // The data layer enforces the page's claims: per-kind rosters, size
    // checks, and fail-the-render guards instead of stale claims.
    assert!(data.contains(r#"file.path("..", "docs", "benchmarks", "big")"#));
    assert!(data.contains("big-file-formatting"));
    assert!(data.contains("markdown_formatters <- c("));
    assert!(data.contains("yaml_formatters <- c("));
    assert!(data.contains("target_roster <- function"));
    assert!(data.contains("big_requested_bytes <- 4000000"));
    assert!(data.contains("directory_files <- 500"));
    for tool in [
        "yamlfmt",
        "yamlfix",
        "dprint-yaml",
        "dprint-markdown",
        "panache",
        "mdformat",
        "prettier",
        "deno-fmt",
    ] {
        assert!(
            data.contains(&format!("\"{tool}\"")),
            "{tool} should be in a comparison roster"
        );
    }

    assert!(rendered.contains("big.md"));
    assert!(rendered.contains("big.yaml"));
    assert!(rendered.contains("big-with-frontmatter.md"));
    assert!(rendered.contains("docs/benchmarks/big"));
    assert!(rendered.contains("MacBook Pro"));
    assert!(rendered.contains("<th style=\"text-align:right;\"> Time </th>"));
    assert!(rendered.contains("<th style=\"text-align:right;\"> Memory </th>"));
    assert!(rendered.contains("<th style=\"text-align:right;\"> User CPU </th>"));
    assert!(rendered.contains("<th style=\"text-align:center;\"> Front matter </th>"));
    for page in [&rendered, &rendered_index] {
        assert!(!page.contains("py-yaml12"));
    }
    assert!(not_found.contains("[Benchmarks](benchmarks.qmd)"));
}

#[test]
fn website_homepage_has_visual_landing_sections() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let index = fs::read_to_string(root.join("index.qmd")).unwrap();
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();

    assert!(index.contains("assets/favicon.svg"));
    assert!(index.contains("hero-shell"));
    assert!(index.contains("terminal-window"));
    assert!(index.contains("workflow-strip"));
    assert!(styles.contains("--yamark-ink"));
    assert!(styles.contains(".hero-shell"));
    assert!(styles.contains(".terminal-window"));
    assert!(styles.contains(".workflow-strip"));
}

#[test]
fn public_docs_keep_unpublished_uv_install_command_in_comments() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let uv_install = concat!("uv tool install ", "yamark");
    for file in ["website/index.qmd", "website/index.html.md"] {
        let contents = fs::read_to_string(root.join(file)).unwrap();
        for line in contents.lines() {
            let line = line.trim_start();
            assert!(
                !line.contains(uv_install) || line.starts_with('#'),
                "{file} should only keep the unpublished uv install command in comments"
            );
        }
    }
}

#[test]
fn website_includes_imported_homepage_and_showcase_content() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let index = fs::read_to_string(root.join("index.qmd")).unwrap();
    let examples = fs::read_to_string(root.join("examples.qmd")).unwrap();
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();

    assert!(index.contains("Yamark formats every layer of a Markdown file"));
    assert!(index.contains("## A quick example"));
    assert!(index.contains("Toggle soft wrap on the Before pane"));
    assert!(index.contains("feature-grid"));
    assert!(!index.contains("## The pitch"));

    assert!(examples.contains("## Markdown-valued YAML scalars"));
    assert!(examples.contains("## Collapse to flow by typing a bracket"));
    assert!(examples.contains("## Recursive Markdown code fences"));
    assert!(examples.contains("## Markdown links, footnotes, and tables"));

    assert!(styles.contains(".before-after"));
    assert!(styles.contains(".showcase-before-after"));
}

#[test]
fn website_documents_user_facing_references() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let reference = fs::read_to_string(root.join("reference.qmd")).unwrap();
    let rendered_reference = fs::read_to_string(root.join("reference.html.md")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("reference.qmd"));
    assert!(not_found.contains("[Reference](reference.qmd)"));

    // The support matrix is documented in the reference page.
    assert!(reference.contains("## What's supported"));
    assert!(
        !reference.contains("lowercase extension ends in `md`"),
        "reference should not imply .cmd and other non-Markdown extensions are supported"
    );
    assert!(
        reference.contains("`.md`, `.qmd`, `.Rmd`, and `.rmd`"),
        "reference should document the exact Markdown-like extensions"
    );
    assert!(
        reference.contains("starts with `ruff`, `air`, `mdformat`, or `prettier`"),
        "reference should document optional configured formatter commands consistently"
    );
    for command in ["`ruff`", "`air`", "`mdformat`", "`prettier`"] {
        assert!(
            reference.contains(command),
            "reference should document optional configured command {command}"
        );
    }

    for option in [
        "--wrap",
        "--canonical",
        "--preserve-footnotes",
        "--line-width",
        "--prose-width",
        "--indent-width",
        "--config",
        "--diagnostics",
        "--compact",
        "--skip-embedded-formatters",
    ] {
        assert!(reference.contains(option), "{option} should be documented");
        assert!(
            rendered_reference.contains(option),
            "{option} should render into reference.html.md"
        );
    }

    assert!(!reference.contains("yamark_help <- function(...)"));
    assert!(!reference.contains("yamark_help()"));
    assert!(!reference.contains("## CLI help"));
    assert!(!rendered_reference.contains("class=\"yamark-cli-help\""));
    assert!(!rendered_reference.contains("## CLI help"));

    for term in [
        "yamark.toml",
        "[format]",
        "[template]",
        "[embedded]",
        "[paths]",
        "editor_options",
        "fmt: compact=false",
        "fmt: canonical=true",
        "#| fmt: skip",
        "hashpipe YAML",
        "Quarto chunk header",
        "missing optional embedded formatter",
        "fmt: off",
        "fmt: on",
        "fmt: markdown",
        "fmt: template.delimiters",
        "fmt: compact",
        "fmt: table",
        "scope=next",
        "scope=from-here",
        "scope=file",
        "Layout repair",
    ] {
        assert!(reference.contains(term), "{term} should be documented");
        assert!(
            rendered_reference.contains(term),
            "{term} should render into reference.html.md"
        );
    }

    for term in [
        "Pandoc citations",
        "Quarto divs",
        "heading attributes",
        "task lists",
        "Reference links",
        "Nested image links",
        "Footnote blocks",
        "Pandoc tables",
        "definition lists",
        "Scalar folding",
        "Flow expansion",
        "Bool/null normalization",
        "Tags and anchors",
        "BOM and line endings",
        "Tab indentation",
    ] {
        assert!(reference.contains(term), "{term} should be documented");
        assert!(
            rendered_reference.contains(term),
            "{term} should render into reference.html.md"
        );
    }
}

#[test]
fn website_documents_cli_help_on_dedicated_page() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let cli_help = fs::read_to_string(root.join("cli-help.qmd")).unwrap();
    let rendered_cli_help = fs::read_to_string(root.join("cli-help.html.md")).unwrap();
    let usage = fs::read_to_string(root.join("usage.qmd")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("cli-help.qmd"));
    assert!(config.contains("text: CLI Help"));
    assert!(usage.contains("[CLI Help](cli-help.qmd)"));
    assert!(not_found.contains("[CLI Help](cli-help.qmd)"));
    assert!(cli_help.contains("title: CLI Help"));
    assert!(cli_help.contains("yamark_bin <- Sys.getenv(\"YAMARK_BIN\", unset = \"\")"));
    assert!(cli_help.contains("yamark_help <- function(...)"));
    for invocation in [
        "yamark_help()",
        "yamark_help(\"format\")",
        "yamark_help(\"git-filter\")",
    ] {
        assert!(
            cli_help.contains(invocation),
            "CLI help page should render generated help for {invocation}"
        );
    }
    for invocation in [
        "yamark_help(\"git-filter\", \"clean\")",
        "yamark_help(\"git-filter\", \"smudge\")",
        "yamark_help(\"git-filter\", \"setup\")",
        "yamark_help(\"git-filter\", \"teardown\")",
    ] {
        assert!(
            !cli_help.contains(invocation),
            "CLI help page should only include top-level --help captures"
        );
    }

    for term in [
        "class=\"yamark-cli-help\"",
        "An ultra-fast YAML and Markdown formatter",
        "Usage:",
        "Commands:",
        "yamark git-filter setup",
        "--markdown-wrap-at-column",
    ] {
        assert!(
            rendered_cli_help.contains(term),
            "{term} should render into generated CLI help"
        );
    }
    assert!(
        !rendered_cli_help.contains("\u{1b}["),
        "rendered CLI help should be converted to HTML spans, not raw ANSI escapes"
    );
}

#[test]
fn website_titles_have_dark_mode_contrast() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();

    assert!(
        styles.contains("body.quarto-dark .quarto-title-block .title"),
        "dark mode should explicitly override page title color"
    );
    assert!(
        styles.contains("body.quarto-dark h1"),
        "dark mode should explicitly override heading title color"
    );
}

#[test]
fn website_documents_editor_and_git_filter_integrations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let editors = fs::read_to_string(root.join("editors.qmd")).unwrap();
    let git_filter = fs::read_to_string(root.join("git-filter.qmd")).unwrap();
    let rendered_editors = fs::read_to_string(root.join("editors.html.md")).unwrap();
    let rendered_git_filter = fs::read_to_string(root.join("git-filter.html.md")).unwrap();
    let usage = fs::read_to_string(root.join("usage.qmd")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    // Editors and Git Filter left the navbar but stay reachable from Usage.
    assert!(not_found.contains("[Editors](editors.qmd)"));
    assert!(not_found.contains("[Git Filter](git-filter.qmd)"));
    assert!(usage.contains("[Editors](editors.qmd)"));
    assert!(usage.contains("[Git Filter](git-filter.qmd)"));

    for term in [
        "VS Code",
        "Positron",
        "Yamark: Format Document",
        "Yamark: Format Selection as Markdown",
        "yamark.useBundledExecutable",
        "yamark.enabledFileExtensions",
        "yamark.extraArguments",
        "yamark.runNextFormatter",
        "yamark.nextFormatterExecutable",
        "Yamark: Show Log",
    ] {
        assert!(editors.contains(term), "{term} should be documented");
        assert!(
            rendered_editors.contains(term),
            "{term} should render into editors.html.md"
        );
    }

    assert!(editors.contains(
        "```json\n{\n  \"yamark.enabledFileExtensions\": [\".md\", \".qmd\", \".yaml\", \".yml\", \".r\", \".py\"],"
    ));
    assert!(rendered_editors.contains(
        "```json\n{\n  \"yamark.enabledFileExtensions\": [\".md\", \".qmd\", \".yaml\", \".yml\", \".r\", \".py\"],"
    ));
    assert!(!editors.contains("```jsonc"));
    assert!(!rendered_editors.contains("```jsonc"));

    for term in [
        "## Experimental status",
        "The Git filter is experimental",
        "may change or be removed",
        "normalize Markdown at the Git boundary",
        "working tree can still be rewritten",
        "yamark git-filter clean",
        "yamark git-filter smudge",
        "yamark git-filter adopt",
        "yamark git-filter join",
        "yamark git-filter check",
        "filter.yamark-md.clean",
        "filter.yamark-md.smudge",
        ".gitattributes",
        "git add --renormalize .",
        "clean(smudge(blob)) == blob",
        "sentence-per-line",
        "column-wrapped",
        "--markdown-wrap-at-column",
        "yamark git-filter teardown",
        "git -c filter.yamark-md.clean=cat add",
        "NEWS.md -filter",
    ] {
        assert!(git_filter.contains(term), "{term} should be documented");
        assert!(
            rendered_git_filter.contains(term),
            "{term} should render into git-filter.html.md"
        );
    }
}

#[test]
fn website_showcase_generates_after_examples_with_yamark() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let examples = fs::read_to_string(root.join("examples.qmd")).unwrap();
    let rendered = fs::read_to_string(root.join("examples.html.md")).unwrap();

    assert!(examples.contains("# fmt: skip file"));
    assert!(examples.contains("<!-- fmt: skip file -->"));
    assert!(rendered.contains("# fmt: skip file"));
    assert!(rendered.contains("<!-- fmt: skip file -->"));
    assert!(examples.contains("showcase_before_after <- function"));
    assert!(examples.contains("system2("));
    assert!(examples.contains("\"yamark\""));
    assert!(examples.contains("showcase_before_after("));
}

#[test]
fn website_keeps_intermediate_markdown_outputs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();

    assert!(config.contains("keep-md: true"));
    for file in [
        "404.html.md",
        "benchmarks.html.md",
        "cli-help.html.md",
        "editors.html.md",
        "examples.html.md",
        "git-filter.html.md",
        "index.html.md",
        "reference.html.md",
        "usage.html.md",
    ] {
        assert!(root.join(file).is_file(), "{file} should be checked in");
    }
}
