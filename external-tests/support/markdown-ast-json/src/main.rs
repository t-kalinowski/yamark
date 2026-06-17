use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use markdown::mdast::{AlignKind, Node, ReferenceKind};
use markdown::{Constructs, ParseOptions, to_mdast};
use serde_json::{Map, Value};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("usage: markdown-ast-json [PATH]...");
        return Ok(());
    }

    let output = match args.as_slice() {
        [] => {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|err| format!("<stdin>: failed to read stdin: {err}"))?;
            markdown_ast_json("<stdin>", &input)?
        }
        [path] => markdown_ast_json_file(&PathBuf::from(path))?,
        paths => {
            let mut values = Vec::with_capacity(paths.len());
            for path in paths {
                let path = PathBuf::from(path);
                let mut object = Map::new();
                object.insert("path".to_owned(), path.display().to_string().into());
                object.insert("ast".to_owned(), markdown_ast_json_file(&path)?);
                values.push(Value::Object(object));
            }
            Value::Array(values)
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|err| format!("failed to serialize JSON: {err}"))?
    );
    Ok(())
}

fn markdown_ast_json_file(path: &Path) -> Result<Value, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|err| format!("{}: failed to read file: {err}", path.display()))?;
    markdown_ast_json(&path.display().to_string(), &input)
}

fn markdown_ast_json(label: &str, input: &str) -> Result<Value, String> {
    let node = to_mdast(input, &parse_options()).map_err(|err| format!("{label}: {err}"))?;
    canonical_node(label, &node)
}

fn parse_options() -> ParseOptions {
    let mut constructs = Constructs::gfm();
    constructs.frontmatter = true;
    ParseOptions {
        constructs,
        ..ParseOptions::gfm()
    }
}

fn canonical_node(label: &str, node: &Node) -> Result<Value, String> {
    let mut object = Map::new();
    match node {
        Node::Root(node) => {
            object.insert("type".to_owned(), "root".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::Blockquote(node) => {
            object.insert("type".to_owned(), "blockquote".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::FootnoteDefinition(node) => {
            object.insert("type".to_owned(), "footnoteDefinition".into());
            object.insert("identifier".to_owned(), node.identifier.clone().into());
            insert_optional_string(&mut object, "label", node.label.as_deref());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::MdxJsxFlowElement(_) => return unsupported(label, "MDX JSX flow element"),
        Node::List(node) => {
            object.insert("type".to_owned(), "list".into());
            object.insert("ordered".to_owned(), node.ordered.into());
            insert_optional_u32(&mut object, "start", node.start);
            object.insert("spread".to_owned(), node.spread.into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::MdxjsEsm(_) => return unsupported(label, "MDX ESM"),
        Node::Toml(_) => return unsupported(label, "TOML front matter"),
        Node::Yaml(node) => {
            object.insert("type".to_owned(), "yaml".into());
            object.insert("marker".to_owned(), "---".into());
            object.insert(
                "value".to_owned(),
                canonical_yaml_value(label, &node.value)?,
            );
        }
        Node::Break(_) => {
            object.insert("type".to_owned(), "break".into());
        }
        Node::InlineCode(node) => {
            object.insert("type".to_owned(), "inlineCode".into());
            object.insert("value".to_owned(), node.value.clone().into());
        }
        Node::InlineMath(node) => {
            object.insert("type".to_owned(), "inlineMath".into());
            object.insert("value".to_owned(), node.value.clone().into());
        }
        Node::Delete(node) => {
            object.insert("type".to_owned(), "delete".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::Emphasis(node) => {
            object.insert("type".to_owned(), "emphasis".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::MdxTextExpression(_) => return unsupported(label, "MDX text expression"),
        Node::FootnoteReference(node) => {
            object.insert("type".to_owned(), "footnoteReference".into());
            object.insert("identifier".to_owned(), node.identifier.clone().into());
            insert_optional_string(&mut object, "label", node.label.as_deref());
        }
        Node::Html(node) => {
            object.insert("type".to_owned(), "html".into());
            object.insert("value".to_owned(), node.value.clone().into());
        }
        Node::Image(node) => {
            object.insert("type".to_owned(), "image".into());
            object.insert("alt".to_owned(), node.alt.clone().into());
            object.insert("url".to_owned(), node.url.clone().into());
            insert_optional_string(&mut object, "title", node.title.as_deref());
        }
        Node::ImageReference(node) => {
            object.insert("type".to_owned(), "imageReference".into());
            object.insert("alt".to_owned(), node.alt.clone().into());
            object.insert("identifier".to_owned(), node.identifier.clone().into());
            object.insert(
                "referenceKind".to_owned(),
                reference_kind(node.reference_kind).into(),
            );
            insert_optional_string(&mut object, "label", node.label.as_deref());
        }
        Node::MdxJsxTextElement(_) => return unsupported(label, "MDX JSX text element"),
        Node::Link(node) => {
            object.insert("type".to_owned(), "link".into());
            object.insert("url".to_owned(), node.url.clone().into());
            insert_optional_string(&mut object, "title", node.title.as_deref());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::LinkReference(node) => {
            object.insert("type".to_owned(), "linkReference".into());
            object.insert("identifier".to_owned(), node.identifier.clone().into());
            object.insert(
                "referenceKind".to_owned(),
                reference_kind(node.reference_kind).into(),
            );
            insert_optional_string(&mut object, "label", node.label.as_deref());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::Strong(node) => {
            object.insert("type".to_owned(), "strong".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::Text(node) => {
            object.insert("type".to_owned(), "text".into());
            object.insert("value".to_owned(), normalize_text(&node.value).into());
        }
        Node::Code(node) => {
            object.insert("type".to_owned(), "code".into());
            let lang = node.lang.as_deref().and_then(normalize_code_lang);
            insert_optional_string(&mut object, "lang", lang.as_deref());
            insert_optional_string(&mut object, "meta", node.meta.as_deref());
            let value = match lang.as_deref() {
                Some("yaml" | "yml") => canonical_yaml_value(label, &node.value)?,
                Some("markdown" | "md") => markdown_ast_json(label, &node.value)?,
                _ => node.value.clone().into(),
            };
            object.insert("value".to_owned(), value);
        }
        Node::Math(node) => {
            object.insert("type".to_owned(), "math".into());
            object.insert("value".to_owned(), node.value.clone().into());
            insert_optional_string(&mut object, "meta", node.meta.as_deref());
        }
        Node::MdxFlowExpression(_) => return unsupported(label, "MDX flow expression"),
        Node::Heading(node) => {
            object.insert("type".to_owned(), "heading".into());
            object.insert("depth".to_owned(), node.depth.into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::Table(node) => {
            object.insert("type".to_owned(), "table".into());
            object.insert(
                "align".to_owned(),
                Value::Array(
                    node.align
                        .iter()
                        .map(|align| align_kind(*align).into())
                        .collect(),
                ),
            );
            insert_children(label, &mut object, &node.children)?;
        }
        Node::ThematicBreak(_) => {
            object.insert("type".to_owned(), "thematicBreak".into());
        }
        Node::TableRow(node) => {
            object.insert("type".to_owned(), "tableRow".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::TableCell(node) => {
            object.insert("type".to_owned(), "tableCell".into());
            insert_children(label, &mut object, &node.children)?;
        }
        Node::ListItem(node) => {
            object.insert("type".to_owned(), "listItem".into());
            object.insert("spread".to_owned(), node.spread.into());
            insert_optional_bool(&mut object, "checked", node.checked);
            insert_children(label, &mut object, &node.children)?;
        }
        Node::Definition(node) => {
            object.insert("type".to_owned(), "definition".into());
            object.insert("identifier".to_owned(), node.identifier.clone().into());
            object.insert("url".to_owned(), node.url.clone().into());
            insert_optional_string(&mut object, "label", node.label.as_deref());
            insert_optional_string(&mut object, "title", node.title.as_deref());
        }
        Node::Paragraph(node) => {
            object.insert("type".to_owned(), "paragraph".into());
            insert_children(label, &mut object, &node.children)?;
        }
    }
    Ok(Value::Object(object))
}

fn insert_children(
    label: &str,
    object: &mut Map<String, Value>,
    children: &[Node],
) -> Result<(), String> {
    object.insert(
        "children".to_owned(),
        Value::Array(
            children
                .iter()
                .map(|child| canonical_node(label, child))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    Ok(())
}

fn insert_optional_string(object: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        object.insert(key.to_owned(), value.to_owned().into());
    }
}

fn insert_optional_bool(object: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        object.insert(key.to_owned(), value.into());
    }
}

fn insert_optional_u32(object: &mut Map<String, Value>, key: &str, value: Option<u32>) {
    if let Some(value) = value {
        object.insert(key.to_owned(), value.into());
    }
}

fn canonical_yaml_value(label: &str, value: &str) -> Result<Value, String> {
    let mut input = tempfile::NamedTempFile::new()
        .map_err(|err| format!("{label}: failed to create YAML temp file: {err}"))?;
    input
        .write_all(value.as_bytes())
        .map_err(|err| format!("{label}: failed to write YAML temp file: {err}"))?;

    let script = yamark_root()?.join("external-tests/support/yaml_suite_value.py");
    let output = Command::new("python3")
        .arg(script)
        .arg("yaml")
        .arg(input.path())
        .output()
        .map_err(|err| {
            format!("{label}: failed to run python3 for YAML canonicalization: {err}")
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{label}: failed to canonicalize YAML: {}",
            stderr.trim()
        ));
    }

    serde_json::from_slice(&output.stdout).map_err(|err| {
        format!(
            "{label}: YAML canonicalizer emitted invalid JSON: {err}: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn yamark_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("YAMARK_ROOT") {
        return Ok(PathBuf::from(root));
    }
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("markdown-ast-json is nested under external-tests/support")
        .to_path_buf())
}

fn unsupported(label: &str, node: &str) -> Result<Value, String> {
    Err(format!(
        "{label}: unsupported Markdown node for canonical JSON: {node}"
    ))
}

fn reference_kind(kind: ReferenceKind) -> &'static str {
    match kind {
        ReferenceKind::Shortcut => "shortcut",
        ReferenceKind::Collapsed => "collapsed",
        ReferenceKind::Full => "full",
    }
}

fn align_kind(kind: AlignKind) -> &'static str {
    match kind {
        AlignKind::Left => "left",
        AlignKind::Right => "right",
        AlignKind::Center => "center",
        AlignKind::None => "none",
    }
}

fn normalize_code_lang(lang: &str) -> Option<String> {
    let lang = lang.trim();
    if lang.is_empty() {
        return None;
    }
    let lang = lang
        .strip_prefix('{')
        .and_then(|lang| lang.strip_suffix('}'))
        .unwrap_or(lang);
    Some(lang.to_ascii_lowercase())
}

fn normalize_text(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut in_whitespace = false;

    for character in value.chars() {
        if character.is_whitespace() {
            if !in_whitespace {
                normalized.push(' ');
            }
            in_whitespace = true;
        } else {
            normalized.push(character);
            in_whitespace = false;
        }
    }

    normalized
}
