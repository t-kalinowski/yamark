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
        r#"description = "A fast formatter for YAML and Markdown.""#,
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
    assert!(pyproject.contains("description = \"A fast formatter for YAML and Markdown.\""));
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
    assert!(readme.contains("Yamark is a fast formatter for YAML and Markdown."));
    assert!(!readme.contains("written in Rust"));
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
    assert!(social_card.contains("A fast formatter for YAML and Markdown."));
    assert!(!social_card.contains("written in Rust"));
    assert!(!social_card.contains("wherever they live"));

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
    assert!(benchmarks.contains("published PyPI release"));
    assert!(benchmarks.contains("benchmark_yamark_version_inline()"));
    assert!(benchmarks.contains("benchmark_commit_inline()"));
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
    assert!(data.contains("benchmark_yamark_version_inline <- function"));
    assert!(data.contains("benchmark_commit_inline <- function"));
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
    assert!(rendered.contains("using the published PyPI release"));
    assert!(rendered.contains(&format!(
        "Yamark `{}`, built from commit",
        env!("CARGO_PKG_VERSION")
    )));
    let commit = rendered
        .split_once("built from commit")
        .unwrap()
        .1
        .split('`')
        .nth(1)
        .unwrap();
    assert_eq!(commit.len(), 12);
    assert!(
        commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );
    assert!(rendered.contains(&format!("docs/benchmarks/big/{commit}")));
    assert!(rendered.contains(&format!("docs/benchmarks/yaml/{commit}")));
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

    assert!(rendered_index.contains(r#"data-benchmark-chart="full-field""#));
    assert!(!rendered_index.contains(r#"data-benchmark-chart="overview""#));
    assert!(rendered.contains(r#"data-benchmark-chart="full-field""#));
    assert!(rendered_index.contains("Every formatter in the checked-in comparison"));
    for formatter in [
        "Yamark", "Panache", "mdformat", "Prettier", "dprint", "Deno", "yamlfmt", "yamlfix",
    ] {
        assert!(rendered_index.contains(formatter));
    }
    assert!(rendered.contains("Next-lowest elapsed"));
    assert!(rendered.contains("Relative time"));
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
    assert!(index.contains("body-classes: yamark-home"));
    assert!(!index.contains("hero-coverage"));
    assert!(!index.contains("hero-specimen"));
    assert!(!index.contains("hero-diff"));
    assert!(!index.contains("terminal-window"));
    assert!(index.contains("workflow-strip"));
    assert!(!index.contains("[Beta]{.status-chip}"));
    assert!(rendered.contains("A fast formatter for YAML and Markdown."));
    assert!(
        rendered.contains("<h1 class=\"hero-thesis\">A fast formatter for YAML and Markdown.</h1>")
    );
    assert!(!rendered.contains("written in Rust"));
    assert!(!rendered.contains("wherever they live"));
    assert!(!rendered.contains("class=\"level2 hero-thesis\""));
    assert!(!rendered.contains("hero-coverage"));
    assert!(!rendered.contains("hero-specimen"));
    assert!(!rendered.contains("<h1 class=\"title\">Yamark</h1>"));
    assert!(!styles.contains(".status-chip"));
    assert!(styles.contains(".yamark-home .quarto-title-block"));
    assert!(styles.contains("display: none"));
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
        "A fast formatter for YAML and Markdown.",
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
    let examples_prose = examples.split_whitespace().collect::<Vec<_>>().join(" ");

    let index_prose = index
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(index.contains("A fast formatter for YAML and Markdown."));
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
    assert!(!index.contains("feature-grid"));
    assert!(!index.contains("### Nested content"));
    assert!(!index.contains("**Dispatch**"));
    assert!(!index.contains("### Nested formatters"));
    assert!(!index.contains("language models"));
    assert!(!index.contains("hero-coverage"));
    assert!(index.contains("## A quick example"));
    assert!(index.contains("Toggle soft wrap on the Before pane"));
    assert!(index.contains("title: Why YAML + Markdown?"));
    assert!(index.contains(
        "The front matter carries fields a program can inspect; the body carries prose that people can edit and review in a diff."
    ));
    assert!(
        index
            .contains("Prompts and agent instructions where metadata sits next to free-form text.")
    );
    assert!(!index.contains("## The pitch"));

    assert!(examples.contains("### Markdown in YAML {#markdown-valued-yaml-scalars}"));
    assert!(examples.contains("Yamark can format Markdown stored in YAML in three ways"));
    assert!(examples.contains("use `!md` as a shorter alias"));
    assert!(examples.contains("instructions: !markdown"));
    assert!(examples.contains("# fmt: markdown\n    instructions: |"));
    assert!(examples.contains("### Source code in YAML"));
    assert!(
        examples_prose.contains(
            "Use `# fmt: python`, `# fmt: r`, or the name of another configured formatter"
        )
    );
    assert!(examples.contains("### Hashpipe YAML in source files"));
    assert!(examples.contains("#| name: demo"));
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

    let markdown_in_yaml = examples
        .find("\n### Markdown in YAML")
        .expect("Markdown-in-YAML example should be present");
    let source_code_in_yaml = examples
        .find("\n### Source code in YAML\n")
        .expect("source-code-in-YAML example should be present");
    let yaml_scalar_presentation = examples
        .find("\n### YAML scalar presentation\n")
        .expect("general YAML scalar example should be present");
    assert!(markdown_in_yaml < source_code_in_yaml);
    assert!(source_code_in_yaml < yaml_scalar_presentation);

    assert!(styles.contains(".before-after"));
    assert!(styles.contains(".showcase-before-after"));
}

#[test]
fn website_documents_json_lines_as_yaml_streams() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = project_root.join("website");
    let reference = fs::read_to_string(root.join("reference-files.qmd")).unwrap();
    let rendered_reference = fs::read_to_string(root.join("reference-files.html.md")).unwrap();
    let examples = fs::read_to_string(root.join("examples.qmd")).unwrap();
    let rendered_examples = fs::read_to_string(root.join("examples.html.md")).unwrap();

    for contents in [&reference, &rendered_reference] {
        let prose = contents.split_whitespace().collect::<Vec<_>>().join(" ");
        for term in [
            "JSON Lines (JSONL) as YAML streams",
            "two or more unmarked, one-line YAML flow-mapping roots",
            "one per physical line",
            "not restricted to strict JSON objects",
            "document-start marker",
            "before each record after the first",
            "--line-width",
        ] {
            assert!(
                prose.contains(term),
                "YAML reference should document {term:?}"
            );
        }
    }

    assert!(examples.contains(r#"stdin_file_path = "records.yaml""#));
    for contents in [&examples, &rendered_examples] {
        let prose = contents.split_whitespace().collect::<Vec<_>>().join(" ");
        for term in [
            "### JSON Lines as a YAML stream",
            "`.yaml` or `.yml`",
            "two or more object records",
            "document-start marker",
            "no leading `---`",
            "records from agent runs",
            "as compact as the width allows",
            "The first record stays in flow style",
            "the second expands only its root mapping",
            "the third also expands `profile` and `events`",
            "Smaller mappings and sequences remain in flow style",
            "not arbitrary JSON values",
        ] {
            assert!(
                prose.contains(term),
                "JSON Lines example should include {term:?}"
            );
        }
    }

    let input = concat!(
        "{\"id\":1,\"profile\":{\"name\":\"planner\",\"active\":true}}\n",
        "{\"id\":2,\"profile\":{\"name\":\"researcher\",\"active\":true},",
        "\"usage\":{\"input_tokens\":640,\"output_tokens\":128}}\n",
        "{\"id\":3,\"profile\":{\"name\":\"reviewer\",\"active\":true,",
        "\"model\":\"gpt-5\",\"region\":\"us\",\"tools\":[\"search\",\"python\"]},",
        "\"events\":[{\"type\":\"tool_call\",",
        "\"tools\":[\"search\",\"open\",\"python\",\"write\"]},",
        "{\"type\":\"usage\",\"tokens\":{\"input\":900,\"output\":240}},",
        "{\"type\":\"completion\",\"status\":\"ok\"}]}\n",
    );
    let expected = ["records.yaml", "records.yml"].map(|stdin_file_path| {
        let output = Command::cargo_bin("yamark")
            .unwrap()
            .args([
                "format",
                "--wrap",
                "72",
                "--stdin-file-path",
                stdin_file_path,
                "--line-width",
                "80",
            ])
            .write_stdin(input)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "JSON Lines example should format through {stdin_file_path}"
        );
        String::from_utf8(output.stdout).unwrap()
    });
    assert_eq!(expected[0], expected[1]);
    assert_eq!(expected[0].matches("\n---\n").count(), 2);
    assert!(expected[0].starts_with("{id: 1, profile: {name: planner, active: true}}\n"));
    let root_expanded = concat!(
        "id: 2\n",
        "profile: {name: researcher, active: true}\n",
        "usage: {input_tokens: 640, output_tokens: 128}\n",
    );
    assert!(expected[0].contains(&format!("\n---\n{root_expanded}---\n")));
    let children_expanded = concat!(
        "id: 3\n",
        "profile:\n",
        "  name: reviewer\n",
        "  active: true\n",
        "  model: gpt-5\n",
        "  region: us\n",
        "  tools: [search, python]\n",
        "events:\n",
        "  - {type: tool_call, tools: [search, open, python, write]}\n",
        "  - {type: usage, tokens: {input: 900, output: 240}}\n",
        "  - {type: completion, status: ok}\n",
    );
    assert!(expected[0].ends_with(children_expanded));
    assert!(
        rendered_examples.contains(expected[0].trim_end()),
        "rendered example should contain current Yamark JSON Lines output"
    );
}

#[test]
fn website_guides_users_through_directives_before_configuration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let usage = fs::read_to_string(root.join("usage.qmd")).unwrap();
    let directives = fs::read_to_string(root.join("directives.qmd")).unwrap();
    let rendered = fs::read_to_string(root.join("directives.html.md")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("text: Directives"));
    assert!(usage.contains("[Directives](directives.qmd)"));
    assert!(not_found.contains("[Directives](directives.qmd)"));
    assert!(directives.contains("[Directive syntax reference](reference-directives.qmd)"));

    for contents in [&directives, &rendered] {
        let whole_file_skip = contents
            .split_once("Skip a whole file by putting the file directive at the top.")
            .expect("directives page should explain whole-file skips")
            .1
            .split_once("## Set file-specific Markdown options")
            .expect("whole-file skip section should end before Markdown options")
            .0;
        assert_eq!(
            whole_file_skip.matches("# fmt: skip file").count(),
            1,
            "the shared YAML, Python, and R form should appear once"
        );
    }

    for term in [
        "The source file is the primary interface",
        "# fmt: markdown",
        "<!-- fmt: ... -->",
        "scope=file",
        "fmt: skip file",
        "fmt: off",
        "format on save",
        "pre-commit",
        "Git Filter",
        "yamark.toml",
        "--wrap",
    ] {
        assert!(
            directives.contains(term),
            "directives page should include {term:?}"
        );
        assert!(
            rendered.contains(term),
            "rendered directives page should include {term:?}"
        );
    }

    let source_examples = [
        directives.find("## YAML files").unwrap(),
        directives.find("## Python files").unwrap(),
        directives.find("## R files").unwrap(),
    ];
    assert!(source_examples[0] < source_examples[1]);
    assert!(source_examples[1] < source_examples[2]);
}

#[test]
fn public_docs_describe_formatting_boundaries_consistently() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    let vscode = fs::read_to_string(root.join("editors/vscode/README.md")).unwrap();
    let vscode_prose = vscode.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(readme.contains("Regions without a supported rewrite stay unchanged."));
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
fn website_organizes_reference_by_lookup_task() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let overview = fs::read_to_string(root.join("reference.qmd")).unwrap();
    let styles = fs::read_to_string(root.join("styles.css")).unwrap();

    let pages = [
        ("reference.qmd", "Reference"),
        ("reference-files.qmd", "Supported files and syntax"),
        ("reference-options.qmd", "Formatting settings"),
        ("reference-config.qmd", "Configuration"),
        ("reference-directives.qmd", "Directive syntax"),
        ("cli-help.qmd", "Command line"),
    ];

    assert!(config.contains("search: true"));
    assert!(config.contains("text: Reference\n        menu:"));
    assert!(styles.contains(".reference-index"));
    assert!(styles.contains(".reference-index {\n    grid-template-columns: 1fr;\n  }"));

    let reference_menu = config
        .split_once("      - text: Reference\n")
        .expect("navbar should contain the Reference menu")
        .1
        .split_once("      - text: Benchmarks\n")
        .expect("Reference menu should end before Benchmarks")
        .0;

    let mut previous = 0;
    for (path, title) in pages {
        let menu_entry = format!("href: {path}");
        let position = reference_menu
            .find(&menu_entry)
            .unwrap_or_else(|| panic!("Reference menu should link to {path}"));
        assert!(
            position >= previous,
            "Reference pages should have a stable order"
        );
        previous = position;

        let source = fs::read_to_string(root.join(path)).unwrap();
        assert!(
            source.contains(&format!("title: {title}")),
            "{path} should have the title {title:?}"
        );

        let rendered = path.replace(".qmd", ".html.md");
        assert!(
            root.join(&rendered).is_file(),
            "{rendered} should be checked in"
        );

        if path != "reference.qmd" {
            assert!(
                overview.contains(&format!("]({path})")),
                "Reference should direct readers to {path}"
            );
        }
    }

    for old_catch_all_section in [
        "## What's supported",
        "## CLI options",
        "## Configuration",
        "## Document Markdown options",
        "## Directives",
        "## Layout repair",
        "## Safety",
    ] {
        assert!(
            !overview.contains(old_catch_all_section),
            "Reference overview should direct readers instead of containing {old_catch_all_section:?}"
        );
    }
}

#[test]
fn website_documents_user_facing_references() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let overview = fs::read_to_string(root.join("reference.qmd")).unwrap();
    let rendered_overview = fs::read_to_string(root.join("reference.html.md")).unwrap();
    let files = fs::read_to_string(root.join("reference-files.qmd")).unwrap();
    let rendered_files = fs::read_to_string(root.join("reference-files.html.md")).unwrap();
    let options = fs::read_to_string(root.join("reference-options.qmd")).unwrap();
    let rendered_options = fs::read_to_string(root.join("reference-options.html.md")).unwrap();
    let config_reference = fs::read_to_string(root.join("reference-config.qmd")).unwrap();
    let rendered_config = fs::read_to_string(root.join("reference-config.html.md")).unwrap();
    let directives = fs::read_to_string(root.join("reference-directives.qmd")).unwrap();
    let rendered_directives =
        fs::read_to_string(root.join("reference-directives.html.md")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("reference.qmd"));
    assert!(not_found.contains("[Reference](reference.qmd)"));
    assert!(overview.contains("#configuration"));
    assert!(rendered_overview.contains("#configuration"));
    for legacy_anchor in [
        "file-types",
        "whats-supported",
        "markdown",
        "yaml",
        "source-files",
        "external-formatters",
        "command-modes",
        "cli-options",
        "format",
        "template",
        "embedded",
        "paths",
        "document-markdown-options",
        "directives",
        "layout-repair",
        "collapse-to-flow-with-or",
        "expand-to-block-with-a-newline",
        "rejection-rules",
        "safety",
    ] {
        let anchor = format!("id=\"{legacy_anchor}\"");
        assert!(
            overview.contains(&anchor),
            "Reference should preserve the old #{legacy_anchor} link"
        );
        assert!(
            rendered_overview.contains(&anchor),
            "rendered Reference should preserve the old #{legacy_anchor} link"
        );
    }
    assert!(directives.starts_with("---\n# fmt: skip file\n"));
    assert!(directives.contains("<!-- fmt: skip file -->"));

    assert!(
        !files.contains("lowercase extension ends in `md`"),
        "reference should not imply .cmd and other non-Markdown extensions are supported"
    );
    assert!(
        files.contains("`.md`, `.qmd`, `.Rmd`, and `.rmd`"),
        "reference should document the exact Markdown-like extensions"
    );

    let files_prose = files.split_whitespace().collect::<Vec<_>>().join(" ");
    let rendered_files_prose = rendered_files
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        files_prose.contains("first argv item is exactly `ruff`, `air`, `mdformat`, or `prettier`"),
        "reference should document optional configured formatter commands precisely"
    );
    for term in [
        "## Files and regions",
        "## Markdown",
        "## YAML",
        "## Python and R source files",
        "## Embedded formatter dispatch",
        "## Layout repair",
        "## Failure behavior",
        "Pandoc citations",
        "Quarto divs",
        "Reference links",
        "Nested image links",
        "Footnote blocks",
        "Pandoc tables",
        "Definition lists",
        "Scalar folding",
        "Core booleans and nulls",
        "Tags and anchors",
        "UTF-8 BOM and line endings",
        "Tab indentation",
        "hashpipe YAML blocks",
        "Long Quarto fence openings",
        "`graphql`, `gql`, `graphqls`, `prettier-graphql`",
        "`postcss`, `pcss`, `prettier-postcss`",
        "`js`, `javascript`, `prettier-js`",
        "`ts`, `typescript`, `prettier-ts`",
        "`bash`, `sh`, `shell`, `zsh`",
        "`powershell`, and `cmd`",
        "supported multiline string literals",
        "same comment prefix",
    ] {
        assert!(files.contains(term), "{term} should be documented");
        assert!(
            rendered_files.contains(term),
            "{term} should render into reference-files.html.md"
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
        assert!(options.contains(option), "{option} should be documented");
        assert!(
            rendered_options.contains(option),
            "{option} should render into reference-options.html.md"
        );
    }

    for term in [
        "yamark.toml",
        "[format]",
        "[template]",
        "[embedded]",
        "[paths]",
        "replace_delimiters",
        "add_delimiters",
        "path_suffix",
        "argv array",
        "accepts exactly one key: the required `formatter` key",
        "requires both `command` and `path_suffix`",
        "At least one complete argv item must be `{path}`",
        "after `replace_delimiters` if both keys are present",
    ] {
        assert!(
            config_reference.contains(term),
            "{term} should be documented"
        );
        assert!(
            rendered_config.contains(term),
            "{term} should render into reference-config.html.md"
        );
    }

    for term in [
        "fmt: compact=false",
        "fmt: canonical=true",
        "#| fmt: skip",
        "Quarto source fence",
        "fmt: off",
        "fmt: on",
        "fmt: markdown",
        "fmt: template.delimiters",
        "fmt: compact",
        "fmt: table",
        "scope=next",
        "scope=from-here",
        "scope=file",
        "empty-valued collection parent",
    ] {
        assert!(directives.contains(term), "{term} should be documented");
        assert!(
            rendered_directives.contains(term),
            "{term} should render into reference-directives.html.md"
        );
    }

    for term in [
        "editor_options",
        "## Document Markdown options",
        "## Interaction rules",
        "Column 72",
        "Structural line width",
        "front matter overrides the corresponding base Markdown settings",
        "Directives then override the corresponding settings within their scope",
        "Unknown front-matter keys and unrecognized values are ignored",
        "rewrites supported `_emphasis_` and `__strong__` spans",
        "Over-width flow collections that cannot be rendered safely in block style may be preserved",
    ] {
        assert!(options.contains(term), "{term} should be documented");
        assert!(
            rendered_options.contains(term),
            "{term} should render into reference-options.html.md"
        );
    }

    for contract in [
        "A missing executable or nonzero exit preserves the target",
        "A successful process that writes to stderr is an error",
        "valid multiline YAML",
        "does not fail solely because the requested flow form is too wide",
        "not restricted to strict JSON objects",
        "For a delegated language, the embedded formatter must succeed",
        "initial consecutive `#|` option block",
        "An explicit directive instead requires a supported target",
        "A multi-file write run is not transactional",
        "Extension matching is ASCII case-insensitive",
        "unsupported extensions are counted as skipped and do not make the run fail",
        "unsupported `--stdin-file-path` is an error",
        "writes are direct rather than atomic",
        "does not promise rollback",
    ] {
        assert!(
            files_prose.contains(contract),
            "supported-files reference should document {contract:?}"
        );
        assert!(
            rendered_files_prose.contains(contract),
            "rendered supported-files reference should document {contract:?}"
        );
    }

    let directives_prose = directives.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(directives_prose.contains(
        "A bare `fmt: template.delimiters` directive uses the next target when placed directly"
    ));
    assert!(directives_prose.contains("requires an explicit scope in every other placement"));
    assert!(directives_prose.contains("anywhere in the initial consecutive `#|` option block"));
    assert!(
        directives_prose
            .contains("Scopes stop at the current parsed document or nested-region boundary")
    );
    assert!(directives_prose.contains(
        "A `scope=file` directive inside one hashpipe YAML block does not span later blocks"
    ));
    assert!(directives_prose.contains(
        "`compact`, `compact=true`, `compact=yes`, `compact=1`, and `compact true` enable compaction"
    ));
    assert!(directives_prose.contains("a Markdown block, including a supported fence"));
    assert!(directives_prose.contains(
        "YAML rejects a targetless `skip`, a stray `on`, a nested `off`, and an `off` without a later `on`"
    ));
    assert!(
        directives_prose.contains("Delimiter arguments must be non-empty double-quoted tokens")
    );
    assert!(directives_prose.contains("`\\\"`, `\\\\`, `\\n`, `\\r`, and `\\t`"));

    let config_prose = config_reference
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for contract in [
        "uses only the nearest file; it does not merge ancestor configs",
        "Each `replace_delimiters` discards delimiters accumulated by earlier layers",
        "Configured embedded-Markdown delimiters do not apply inside Python f-strings",
    ] {
        assert!(
            config_prose.contains(contract),
            "configuration reference should document {contract:?}"
        );
    }

    let options_prose = options.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(options_prose.contains(
        "For YAML input, emits trace counters; it also emits notes for skipped or failing optional embedded formatters"
    ));
    assert!(
        options_prose
            .contains("`wrap`, `format`, `true`, `yes`, and `1` format footnote definitions")
    );
    assert!(options_prose.contains("`preserve`, `none`, `false`, `no`, and `0` preserve them"));

    assert!(rendered_files_prose.contains("leaves unsupported regions unchanged"));
}

#[test]
fn website_documents_command_line_reference() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("website");
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let cli_help = fs::read_to_string(root.join("cli-help.qmd")).unwrap();
    let rendered_cli_help = fs::read_to_string(root.join("cli-help.html.md")).unwrap();
    let usage = fs::read_to_string(root.join("usage.qmd")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("text: Command line"));
    assert!(usage.contains("[Command line](cli-help.qmd)"));
    let usage_prose = usage.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        usage_prose.contains(
            "generated `--help` for Yamark, `format`, and the `git-filter` command group"
        )
    );
    assert!(!usage_prose.contains("generated `--help` for each command"));
    assert!(not_found.contains("[Command line](cli-help.qmd)"));
    assert!(cli_help.contains("title: Command line"));
    assert!(cli_help.contains("## Modes, output, and status"));
    assert!(cli_help.contains("## Generated help"));
    assert!(cli_help.contains("source(\"_yamark-build.R\")"));
    assert!(cli_help.contains("yamark_help <- function(...)"));
    assert!(cli_help.contains("NO_COLOR="));
    for contents in [&cli_help, &rendered_cli_help] {
        let prose = contents.split_whitespace().collect::<Vec<_>>().join(" ");
        for contract in [
            "`yamark format PATHS`",
            "`yamark format --check PATHS`",
            "`yamark format --diff PATHS`",
            "`yamark format --stdin-file-path PATH`",
            "When `PATHS` is omitted, Yamark uses the current directory (`.`)",
            "An unsupported `--stdin-file-path` is an error",
            "Unified diffs",
            "Formatted content only",
            "exits `1`",
            "exits `2`",
        ] {
            assert!(
                prose.contains(contract),
                "command-line reference should document {contract:?}"
            );
        }
    }
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
        "A fast formatter for YAML and Markdown.",
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
    assert!(!rendered_cli_help.contains("written in Rust"));
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

    for file in ["index.qmd", "examples.qmd", "cli-help.qmd"] {
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
}

#[test]
fn public_docs_do_not_advertise_verify() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "README.md",
        "website/cli-help.html.md",
        "website/examples.qmd",
        "website/index.qmd",
        "website/reference-config.qmd",
        "website/reference-directives.qmd",
        "website/reference-files.qmd",
        "website/reference-options.qmd",
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
    let config = fs::read_to_string(root.join("_quarto.yml")).unwrap();
    let editors = fs::read_to_string(root.join("editors.qmd")).unwrap();
    let git_filter = fs::read_to_string(root.join("git-filter.qmd")).unwrap();
    let rendered_editors = fs::read_to_string(root.join("editors.html.md")).unwrap();
    let rendered_git_filter = fs::read_to_string(root.join("git-filter.html.md")).unwrap();
    let usage = fs::read_to_string(root.join("usage.qmd")).unwrap();
    let not_found = fs::read_to_string(root.join("404.qmd")).unwrap();

    assert!(config.contains("text: Editors"));
    assert!(!config.contains("text: Home"));
    assert!(!config.contains("text: Releases"));

    // Git Filter stays out of the navbar but remains reachable from Usage.
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
        "Yamark's experimental Git filter",
        "may change or be removed",
        "sentence-per-line Markdown in Git",
        "other tools see the",
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
        "directives.html.md",
        "editors.html.md",
        "examples.html.md",
        "git-filter.html.md",
        "index.html.md",
        "reference-config.html.md",
        "reference-directives.html.md",
        "reference-files.html.md",
        "reference-options.html.md",
        "reference.html.md",
        "usage.html.md",
    ] {
        assert!(root.join(file).is_file(), "{file} should be checked in");
    }

    for stem in [
        "reference",
        "reference-config",
        "reference-files",
        "reference-options",
    ] {
        let source = fs::read_to_string(root.join(format!("{stem}.qmd"))).unwrap();
        let rendered = fs::read_to_string(root.join(format!("{stem}.html.md"))).unwrap();
        assert_eq!(source, rendered, "{stem}.html.md should match its source");
    }
}
