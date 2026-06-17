use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::mem::size_of;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions};
use yamark::core::parser::format_source_report;
use yamark::core::source::{Line, SourceSpan, Span};
use yamark::core::yaml_model::YamlAstNode;
use yamark::plugins::PluginRegistry;

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static COUNT_THREAD_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if count_thread_allocations() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if count_thread_allocations() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(new_size.saturating_sub(layout.size()), Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

fn set_count_thread_allocations(enabled: bool) {
    COUNT_THREAD_ALLOCATIONS.with(|count| count.set(enabled));
}

fn count_thread_allocations() -> bool {
    COUNT_THREAD_ALLOCATIONS.with(Cell::get)
}

#[test]
fn yaml_ast_node_layout_stays_semantic_only() {
    assert_eq!(
        size_of::<SourceSpan<'_>>(),
        8,
        "SourceSpan is {} bytes",
        size_of::<SourceSpan<'_>>()
    );
    assert_eq!(
        size_of::<Option<SourceSpan<'_>>>(),
        8,
        "Option<SourceSpan> is {} bytes",
        size_of::<Option<SourceSpan<'_>>>()
    );
    assert!(
        size_of::<SourceSpan<'_>>() < size_of::<Span>(),
        "SourceSpan is {} bytes; Span is {} bytes",
        size_of::<SourceSpan<'_>>(),
        size_of::<Span>()
    );
    assert!(
        size_of::<YamlAstNode>() <= 144,
        "YamlAstNode is {} bytes",
        size_of::<YamlAstNode>()
    );
}

#[test]
fn source_line_layout_stays_compact() {
    assert!(
        size_of::<Line>() <= 24,
        "Line is {} bytes",
        size_of::<Line>()
    );
}

#[test]
fn plugin_registry_default_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    black_box(PluginRegistry::default());

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    for _ in 0..100 {
        black_box(PluginRegistry::default());
    }
    set_count_thread_allocations(false);

    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 150,
        "default plugin registry setup allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn config_default_path_application_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let config = Config::default();
    let path = Path::new("/tmp/yamark/default/config.yaml");
    black_box(config.for_formatted_path(path));

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    for _ in 0..100 {
        black_box(config.for_formatted_path(path));
    }
    set_count_thread_allocations(false);

    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 1_900,
        "default config path application allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_flow_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = flow_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 59_800,
        "flow table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn markdown_showcase_like_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = markdown_showcase_like_input(120);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup = format_source_report(
        FileKind::Markdown,
        input.clone(),
        options,
        &config,
        &plugins,
    )
    .unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Markdown, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 19_000,
        "showcase-like Markdown formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_quoted_scalar_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = quoted_scalar_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 66_300,
        "quoted scalar table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_quoted_scalar_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_quoted_scalar_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 89_500,
        "escaped quoted scalar table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_plain_string_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_plain_string_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 89_800,
        "escaped plain string table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_string_tag_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_string_tag_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 113_000,
        "escaped string tag table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_core_tag_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_core_tag_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 77_600,
        "escaped core tag table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_int_tag_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_int_tag_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 74_200,
        "escaped int tag table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_float_tag_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_float_tag_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 47_200,
        "escaped float tag table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_escaped_core_like_string_table_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = escaped_core_like_string_table_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert_eq!(formatted.output.lines().count(), 401);
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 91_000,
        "escaped core-like string table formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_json_like_flow_scalar_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = json_like_flow_scalar_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert!(formatted.output.contains("service-00000"));
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 11_000,
        "JSON-like flow scalar formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_block_plain_scalar_formatting_keeps_allocation_count_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = block_plain_scalar_input(400);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert!(formatted.output.contains("enabled: true"));
    assert!(
        ALLOCATIONS.load(Ordering::Relaxed) <= 52_000,
        "block plain scalar formatting allocated {} times",
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_showcase_like_formatting_keeps_allocated_bytes_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = yaml_showcase_like_input(1_000);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert!(formatted.output.contains("benchmark comment 001000.01"));
    assert!(
        ALLOCATED_BYTES.load(Ordering::Relaxed) <= 10_100_000,
        "showcase-like YAML formatting allocated {} bytes",
        ALLOCATED_BYTES.load(Ordering::Relaxed)
    );
}

#[test]
fn yaml_many_line_formatting_keeps_allocated_bytes_bounded() {
    let _lock = ALLOCATION_LOCK.lock().unwrap();
    let input = many_line_yaml_input(50_000);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions::default();

    let warmup =
        format_source_report(FileKind::Yaml, input.clone(), options, &config, &plugins).unwrap();
    assert!(warmup.changed);

    ALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    set_count_thread_allocations(true);
    let formatted = format_source_report(FileKind::Yaml, input, options, &config, &plugins);
    set_count_thread_allocations(false);

    let formatted = formatted.unwrap();
    assert!(formatted.changed);
    assert!(formatted.output.contains("item_49999"));
    assert!(
        ALLOCATED_BYTES.load(Ordering::Relaxed) <= 197_000_000,
        "many-line YAML formatting allocated {} bytes",
        ALLOCATED_BYTES.load(Ordering::Relaxed)
    );
}

fn flow_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: item_{index}, type: !!str \"boolean\", default: FALSE}}\n"
        ));
    }
    input
}

fn many_line_yaml_input(rows: usize) -> String {
    let mut input = String::new();
    for index in 0..rows {
        input.push_str(&format!(
            "- name: item_{index:05}\n  enabled: TRUE\n  count: 123\n"
        ));
    }
    input
}

fn markdown_showcase_like_input(blocks: usize) -> String {
    let mut input = String::from("# Showcase\n\n");
    for index in 0..blocks {
        input.push_str(&format!(
            "This paragraph {index} carries enough prose to wrap across several lines and includes [a short link](https://example.com/short/{index:04}) plus [a long unwrappable reference](https://example.com/pathological/segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}) before the sentence ends.\n\n"
        ));
        if index % 4 == 0 {
            input.push_str(&format!(
                "- top item {index:04} carries lorem prose and [a long unwrappable reference](https://example.com/pathological/segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}).\n- top item {next:04} carries lorem prose and [a short reference](https://example.com/short/{next:04}).\n  - nested item {index:04} carries lorem prose and [a long unwrappable reference](https://example.com/pathological/segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}-segment-{index:04}).\n\n",
                next = index + 1
            ));
        }
    }
    input
}

fn block_plain_scalar_input(rows: usize) -> String {
    let mut input = String::from("settings:\n");
    for index in 0..rows {
        input.push_str(&format!(
            "  item_{index:03}:\n    name: worker-{index:03}\n    replicas: 3\n    enabled: TRUE\n    memory: 128Mi\n    region: us-east-{region}\n",
            region = index % 4
        ));
    }
    input
}

fn yaml_showcase_like_input(nodes: usize) -> String {
    let mut input = String::new();
    for index in 1..=nodes {
        input.push_str(&format!("# benchmark comment {index:06}.01\n"));
        input.push_str("- str:\n");
        input
            .push_str("  - Lorem ipsum dolor sit amet, vel accumsan vitae faucibus ultrices leo\n");
        input.push_str("  - neque? Et cursus lacinia, ut, sit donec facilisi eu interdum. Dui\n");
        input.push_str(
            "  - ipsum, vitae ligula commodo convallis ac sed nunc. Ipsum at nec lacus\n",
        );
        input.push_str("  - eros suscipit vitae.\n");
        input.push_str("  block_str: |\n");
        input.push_str("    lorem\n");
        input.push_str("     ipsum\n");
        input.push_str("    dolor\n");
        input.push_str("  bools:\n");
        input.push_str("  - TRUE\n");
        input.push_str("  - FALSE\n");
        input.push_str("  ints:\n");
        input.push_str("  - 123\n");
        input.push_str("  - -123\n");
        input.push_str("  floats:\n");
        input.push_str("  - 123.456\n");
        input.push_str("  - -123.456\n");
        input.push_str("  \"null\": ~\n");
    }
    input
}

fn json_like_flow_scalar_input(rows: usize) -> String {
    let mut input = String::from("[");
    for index in 0..rows {
        if index > 0 {
            input.push(',');
        }
        input.push_str(&format!(
            "{{\"id\":\"service-{index:05}\",\"enabled\":true,\"count\":123,\"ratio\":0.125,\"empty\":null,\"labels\":[\"alpha\",\"beta\",\"gamma\"],\"nested\":{{\"owner\":\"team-{index:03}\",\"mode\":\"safe\"}}}}"
        ));
    }
    input.push_str("]\n");
    input
}

fn quoted_scalar_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: \"item_{index}\", label: \"alpha beta\", mode: \"safe\"}}\n"
        ));
    }
    input
}

fn escaped_quoted_scalar_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: \"item\\x20{index}\", label: \"alpha\\x20beta\", mode: \"safe\\x20mode\"}}\n"
        ));
    }
    input
}

fn escaped_plain_string_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: \"item\\x2d{index}\", label: \"alpha\\x2dbeta\", mode: \"safe\\x2dmode\"}}\n"
        ));
    }
    input
}

fn escaped_string_tag_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: item_{index}, label: !!str \"alpha\\tbeta\", mode: !!str \"safe\\tmode\"}}\n"
        ));
    }
    input
}

fn escaped_core_tag_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: item_{index}, flag: !!bool \"TR\\x55E\", empty: !!null \"Nu\\x6cl\"}}\n"
        ));
    }
    input
}

fn escaped_int_tag_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: item_{index}, count: !!int \"\\x31_000\", delta: !!int \"\\x2d42\"}}\n"
        ));
    }
    input
}

fn escaped_float_tag_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: item_{index}, ratio: !!float \"\\x33.14\", limit: !!float \"\\x31.0e\\x2d3\"}}\n"
        ));
    }
    input
}

fn escaped_core_like_string_table_input(rows: usize) -> String {
    let mut input = String::from("# fmt: table\n");
    for index in 0..rows {
        input.push_str(&format!(
            "- {{name: item_{index}, flag: \"TR\\x55E\", mode: !!str \"TR\\x55E\", count: \"\\x31_000\", label: !!str \"\\x33.14\"}}\n"
        ));
    }
    input
}
