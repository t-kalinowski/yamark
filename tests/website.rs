use assert_cmd::Command;
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
        r#"description = "An extremely fast formatter for YAML and Markdown, written in Rust.""#,
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
    assert!(pyproject.contains(
        "description = \"An extremely fast formatter for YAML and Markdown, written in Rust.\""
    ));
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
    assert!(
        ci.contains("r-lib/actions/setup-r@v2"),
        "CI should install R because Rust tests invoke Rscript"
    );
    assert!(
        ci.contains("install.packages"),
        "CI should install R packages used by benchmark tests"
    );
    for package in ["stringi", "yaml12", "jsonlite", "knitr"] {
        assert!(
            ci.contains(package),
            "CI should install R package {package}"
        );
    }
    assert!(
        ci.contains("uv tool install ruff"),
        "CI should install ruff because Rust tests invoke the default Python formatter"
    );

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
fn package_versions_stay_in_sync() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let pyproject = fs::read_to_string(root.join("pyproject.toml")).unwrap();
    let extension = fs::read_to_string(root.join("editors/vscode/package.json")).unwrap();
    let cargo_lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
    let uv_lock = fs::read_to_string(root.join("uv.lock")).unwrap();

    let cargo: toml::Value = toml::from_str(&cargo).unwrap();
    let pyproject: toml::Value = toml::from_str(&pyproject).unwrap();
    let extension: serde_json::Value = serde_json::from_str(&extension).unwrap();
    let version = cargo["package"]["version"].as_str().unwrap();

    assert_eq!(pyproject["project"]["version"].as_str(), Some(version));
    assert_eq!(extension["version"].as_str(), Some(version));
    let package = format!("name = \"yamark\"\nversion = \"{version}\"");
    assert!(cargo_lock.contains(&package));
    assert!(uv_lock.contains(&package));
}

#[test]
fn release_workflow_publishes_python_package() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let release = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
    let readme = fs::read_to_string(root.join("README.md")).unwrap();

    for target in [
        "x86_64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
    ] {
        assert!(
            release.contains(target),
            "release workflow should build a Python wheel for {target}"
        );
    }
    for contract in [
        "PyO3/maturin-action@v1",
        "manylinux: \"2014\"",
        "--release --locked --compatibility pypi",
        "command: sdist",
        "uv tool install --no-index --find-links dist yamark",
        r#""$(uv tool dir --bin)/yamark" --help"#,
        "Join-Path (uv tool dir --bin)",
        "needs: [build, wheels, sdist]",
        "pattern: yamark-${{ github.ref_name }}-*",
        "needs: [release, wheels, sdist]",
        "pattern: python-*",
        "environment: pypi",
        "id-token: write",
        "pypa/gh-action-pypi-publish@release/v1",
        "packages-dir: dist/",
    ] {
        assert!(
            release.contains(contract),
            "release workflow should include {contract}"
        );
    }
    assert!(
        readme.contains("uvx yamark format"),
        "README should document running the published package"
    );
}

#[test]
fn github_pages_workflow_publishes_website() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pages = fs::read_to_string(root.join(".github/workflows/pages.yml")).unwrap();

    assert!(pages.contains("branches: [main]"));
    assert!(pages.contains("workflow_dispatch:"));
    assert!(pages.contains("contents: read"));
    assert!(pages.contains("pages: write"));
    assert!(pages.contains("id-token: write"));
    assert!(pages.contains("actions/configure-pages@v5"));
    assert!(pages.contains("quarto-dev/quarto-actions/setup@v2"));
    for package in ["jsonlite", "knitr", "rmarkdown", "htmltools", "fansi"] {
        assert!(
            pages.contains(package),
            "Pages workflow should install R package {package}"
        );
    }
    assert!(pages.contains("cargo build --release --bin yamark"));
    assert!(pages.contains("YAMARK_BIN="));
    assert!(pages.contains("quarto render website"));
    assert!(pages.contains("actions/upload-pages-artifact@v3"));
    assert!(pages.contains("path: website/_site"));
    assert!(pages.contains("actions/deploy-pages@v4"));
}

#[test]
fn website_social_images_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    assert!(config.contains("image: assets/social-card.svg"));
    assert!(!config.contains("image: assets/favicon.svg"));

    let social_card = fs::read_to_string(root.join("assets/social-card.svg")).unwrap();
    assert!(social_card.contains(r#"viewBox="0 0 1200 630""#));
    assert!(social_card.contains("Markdown and YAML are source files too."));

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
    assert!(benchmarks.contains("## Methodology"));
    assert!(!benchmarks.contains("How to read these results"));
    assert!(benchmarks.contains("Reproducing"));

    // One comparison per input kind, each with its own native-CLI roster.
    assert!(benchmarks.contains("## At a glance"));
    assert!(benchmarks.contains("## Detailed results"));
    assert!(benchmarks.contains("\n### Markdown\n"));
    assert!(benchmarks.contains("\n### YAML\n"));
    assert!(benchmarks.contains("\n### Markdown + front matter\n"));
    assert!(benchmarks.contains("\n### Directory\n"));
    assert!(benchmarks.contains("tools/bench/big.py"));
    assert!(benchmarks.contains("--files 500 --items 540"));
    assert!(benchmarks.contains("MacBook Pro"));
    assert!(benchmarks.contains("total user CPU time"));
    assert!(!benchmarks.contains("single-core comparison"));

    // The page must say, in visible prose, why the lint fixers are outside
    // this comparison instead of burying the scope choice in a comment.
    assert!(benchmarks.contains("pymarkdown"));
    assert!(benchmarks.contains("markdownlint-cli2"));
    assert!(benchmarks.contains("formatter-CLI comparison"));

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
    assert!(rendered.contains("<th style=\"text-align:right;\"> Wall time </th>"));
    assert!(rendered.contains("<th style=\"text-align:right;\"> Peak RSS </th>"));
    assert!(rendered.contains("<th style=\"text-align:right;\"> User CPU time </th>"));
    assert!(rendered.contains("<th style=\"text-align:center;\"> Front matter </th>"));
    assert!(!rendered.contains("> Throughput </th>"));
    assert!(!rendered.contains("> vs yamark </th>"));
    for page in [&rendered, &rendered_index] {
        assert!(page.contains("lowest elapsed time in all four workloads"));
        assert!(!page.contains("next-fastest"));
        assert!(!page.contains("py-yaml12"));
    }
    assert!(not_found.contains("[Benchmarks](benchmarks.qmd)"));
}

#[test]
fn website_presents_benchmarks_with_data_driven_visuals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let data = fs::read_to_string(root.join("_benchmark-data.R")).unwrap();
    let charts = fs::read_to_string(root.join("benchmark-charts.js")).unwrap();
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();
    let rendered = fs::read_to_string(root.join("benchmarks.html.md")).unwrap();
    let rendered_index = fs::read_to_string(root.join("index.html.md")).unwrap();

    assert!(config.contains("benchmark-charts.js"));
    assert!(data.contains("benchmark_summary_rows"));
    assert!(data.contains("benchmark_full_field_rows"));
    assert!(data.contains("write_benchmark_chart"));

    assert!(rendered_index.contains(r#"data-benchmark-chart="overview""#));
    assert!(rendered.contains(r#"data-benchmark-chart="full-field""#));
    assert!(rendered.contains("Next-lowest elapsed"));
    assert!(rendered.contains("Peer / Yamark"));
    assert!(rendered.contains("Output note"));
    assert!(rendered.contains("benchmark-summary-table"));
    assert!(rendered_index.contains("Exact values:"));

    assert!(charts.contains("Math.log10"));
    assert!(charts.contains("ResizeObserver"));
    assert!(charts.contains(r#"setAttribute("role", "img")"#));
    assert!(charts.contains("aria-labelledby"));
    assert!(charts.contains("svgNode(\"title\""));
    assert!(charts.contains("svgNode(\"desc\""));
    assert!(styles.contains(".benchmark-chart"));
    assert!(styles.contains(".benchmark-full-field-grid"));
    assert!(styles.contains("@container benchmark-summary"));
}

#[test]
fn website_homepage_has_visual_landing_sections() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let index = fs::read_to_string(root.join("index.qmd")).unwrap();
    let rendered = fs::read_to_string(root.join("index.html.md")).unwrap();
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();

    assert!(index.contains("assets/favicon.svg"));
    assert!(index.contains("hero-shell"));
    assert!(index.contains("hero-thesis"));
    assert!(index.contains("hero-command"));
    assert!(!index.contains("hero-coverage"));
    assert!(!index.contains("hero-specimen"));
    assert!(!index.contains("hero-diff"));
    assert!(!index.contains("terminal-window"));
    assert!(index.contains("workflow-strip"));
    assert!(!index.contains("[Beta]{.status-chip}"));
    assert!(rendered.contains("Format Markdown and YAML wherever they live."));
    assert!(
        rendered.contains(
            "<h2 class=\"hero-thesis\">Format Markdown and YAML wherever they live.</h2>"
        )
    );
    assert!(!rendered.contains("class=\"level2 hero-thesis\""));
    assert!(!rendered.contains("hero-coverage"));
    assert!(!rendered.contains("hero-specimen"));
    assert!(!styles.contains(".status-chip"));
    assert!(styles.contains("--yamark-ink"));
    assert!(styles.contains(".hero-shell"));
    assert!(styles.contains(".hero-thesis"));
    assert!(styles.contains(".hero-command"));
    assert!(!styles.contains(".hero-coverage"));
    assert!(!styles.contains(".hero-specimen"));
    assert!(!styles.contains(".hero-diff"));
    assert!(!styles.contains(".terminal-window"));
    assert!(styles.contains(".workflow-strip"));

    let mobile = styles
        .split("@media (max-width: 560px)")
        .nth(1)
        .expect("website should have narrow-screen styles");
    assert!(mobile.contains(".hero-command"));
    assert!(mobile.contains("flex-direction: column"));
}

#[test]
fn website_homepage_leads_with_purpose_and_scope() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let index = fs::read_to_string(root.join("index.qmd")).unwrap();
    let rendered = fs::read_to_string(root.join("index.html.md")).unwrap();
    let rendered_prose = rendered.split_whitespace().collect::<Vec<_>>().join(" ");

    for text in [
        "Format Markdown and YAML wherever they live.",
        "uvx yamark format",
    ] {
        assert!(rendered.contains(text), "homepage should include {text:?}");
    }
    assert!(rendered_prose.contains(
        "Yamark formats whole files and embedded content with the consistency we expect from code, keeping source readable and changes easy to review."
    ));
    assert!(!rendered.contains("<ul class=\"hero-coverage\""));
    assert!(!rendered.contains("documentation, configuration, prompts, and agent instructions"));
    assert!(!index.contains("label: hero-example"));
    assert!(!index.contains("emit_hero_specimen"));
    assert!(!rendered.contains("files scanned"));
}

#[test]
fn website_code_blocks_reset_quarto_wrapper_margins() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();
    let styles = styles.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(styles.contains(".code-copy-outer-scaffold div.sourceCode { border: 0; margin: 0; }"));
}

#[test]
fn public_docs_show_pypi_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in ["README.md", "website/usage.qmd", "website/usage.html.md"] {
        let contents = fs::read_to_string(root.join(file)).unwrap();
        for command in ["uvx yamark format", "uv tool install yamark"] {
            assert!(
                contents.contains(command),
                "{file} should document {command}"
            );
        }
    }
}

#[test]
fn website_includes_homepage_and_examples_content() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let index = fs::read_to_string(root.join("index.qmd")).unwrap();
    let examples = fs::read_to_string(root.join("examples.qmd")).unwrap();
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();

    let index_prose = index
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(index.contains("Format Markdown and YAML wherever they live."));
    assert!(index_prose.contains(
        "Yamark formats whole files and embedded content with the consistency we expect from code, keeping source readable and changes easy to review."
    ));
    assert!(!index.contains("Markdown and YAML are source files too."));
    assert!(!index.contains("they hold documentation, configuration"));
    assert!(index_prose.contains(
        "Here is a Markdown file with YAML front matter, a long paragraph, and a nested list. Yamark formats the YAML and Markdown in one pass. The first pane shows the input; the second shows what `yamark format` writes back."
    ));
    assert!(!index.contains("YAML and Markdown often share a file."));
    assert!(index.contains("**Recurse**"));
    assert!(index.contains("### Nested content"));
    assert!(!index.contains("**Dispatch**"));
    assert!(!index.contains("### Nested formatters"));
    assert!(!index.contains("language models"));
    assert!(!index.contains("hero-coverage"));
    assert!(index.contains("## A quick example"));
    assert!(index.contains("Toggle soft wrap on the Before pane"));
    assert!(index.contains("feature-grid"));
    assert!(!index.contains("## The pitch"));

    assert!(examples.contains("### Markdown-valued YAML scalars"));
    assert!(examples.contains("REVIEW_PROMPT"));
    assert!(examples.contains("agents:"));
    assert!(examples.contains("### Collapse to flow by typing a bracket"));
    assert!(examples.contains("### Recursive Markdown code fences"));
    let markdown_documents = examples
        .find("\n## Markdown links, footnotes, and tables\n")
        .expect("Markdown document examples should have a top-level section");
    let markdown_in_source = examples
        .find("\n## Markdown in source files\n")
        .expect("source-file examples should have a top-level section");
    assert!(markdown_documents < markdown_in_source);

    assert!(styles.contains(".before-after"));
    assert!(styles.contains(".showcase-before-after"));
}

#[test]
fn public_docs_describe_formatting_boundaries_consistently() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    let vscode = fs::read_to_string(root.join("editors/vscode/README.md")).unwrap();
    let vscode_prose = vscode.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(readme.contains("It rewrites supported regions and preserves unsupported input."));
    assert!(readme.contains("For supported embedded code, Yamark can also call"));
    for statement in [
        "Yamark formats `#|` hashpipe YAML comment blocks and explicitly marked targets",
        "It preserves the surrounding source code.",
        "Embedded target formatting and whole-document chaining have different scopes.",
        "`yamark.nextFormatterExecutable` instead receives Yamark's full document output",
    ] {
        assert!(
            vscode_prose.contains(statement),
            "VS Code documentation should explain: {statement}"
        );
    }
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
    assert!(cli_help.contains("source(\"_yamark-build.R\")"));
    assert!(cli_help.contains("yamark_help <- function(...)"));
    assert!(cli_help.contains("NO_COLOR="));
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
        "An extremely fast formatter for YAML and Markdown",
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
    for style in [
        "color: #5555FF",
        "color: #00BBBB",
        "color: #555555",
        "font-weight: bold",
    ] {
        assert!(
            rendered_cli_help.contains(style),
            "rendered CLI help should contain {style}"
        );
    }
}

#[test]
fn website_renders_examples_with_one_current_yamark_binary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let helper = fs::read_to_string(root.join("_yamark-build.R")).unwrap();

    assert!(helper.contains("YAMARK_BIN"));
    assert!(helper.contains("cargo"));
    assert!(helper.contains("--release"));
    assert!(helper.contains(r#"file.path("..", "target", "release", yamark_exe)"#));
    assert!(helper.contains("yamark_format <- function"));

    for file in ["index.qmd", "examples.qmd", "reference.qmd", "cli-help.qmd"] {
        let contents = fs::read_to_string(root.join(file)).unwrap();
        assert!(
            contents.contains("source(\"_yamark-build.R\")"),
            "{file} should resolve Yamark through the shared site helper"
        );
        assert!(
            !contents.contains("target\", \"debug"),
            "{file} should not select a debug build"
        );
    }

    let reference = fs::read_to_string(root.join("reference.qmd")).unwrap();
    let rendered_reference = fs::read_to_string(root.join("reference.html.md")).unwrap();
    assert!(reference.contains("reference_before_after("));
    let output = Command::cargo_bin("yamark")
        .unwrap()
        .args(["format", "--wrap", "72", "--stdin-file-path", "layout.yaml"])
        .write_stdin("tags: [\n  - yaml\n  - markdown\n  - docs\n")
        .output()
        .unwrap();
    assert!(output.status.success());
    let expected = String::from_utf8(output.stdout).unwrap();
    assert!(rendered_reference.contains(expected.trim_end()));
    assert!(!rendered_reference.contains("tags: [llm, authoring, formats]"));
}

#[test]
fn public_docs_do_not_advertise_verify() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "README.md",
        "website/cli-help.html.md",
        "website/examples.qmd",
        "website/index.qmd",
        "website/reference.qmd",
        "website/usage.qmd",
    ] {
        let contents = fs::read_to_string(root.join(file)).unwrap();
        assert!(
            !contents.contains("--verify"),
            "{file} should not advertise the internal verification option"
        );
    }
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
        "Yamark: Open Filtered Working Tree Diff",
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
        "sentence-per-line Markdown in Git",
        "tools therefore see the column-wrapped form",
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
        "Yamark: Open Filtered Working Tree Diff",
        "git cat-file --filters",
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
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = project_root.join("website");
    let examples = fs::read_to_string(root.join("examples.qmd")).unwrap();
    let helper = fs::read_to_string(root.join("_yamark-build.R")).unwrap();
    let rendered = fs::read_to_string(root.join("examples.html.md")).unwrap();
    let pages = fs::read_to_string(project_root.join(".github/workflows/pages.yml")).unwrap();

    assert!(examples.contains("# fmt: skip file"));
    assert!(examples.contains("<!-- fmt: skip file -->"));
    assert!(rendered.contains("# fmt: skip file"));
    assert!(rendered.contains("<!-- fmt: skip file -->"));
    assert!(examples.contains("showcase_before_after <- function"));
    assert!(examples.contains("yamark_format("));
    assert!(examples.contains("showcase_before_after("));
    assert!(helper.contains("system2("));
    assert!(helper.contains("yamark_bin"));

    for command in [
        "uv tool install ruff==0.16.1",
        "npm install --global prettier@3.8.3",
    ] {
        assert!(
            pages.contains(command),
            "Pages should install the formatter used by generated examples: {command}"
        );
    }
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
