use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions, MarkdownWrap};
use yamark::core::parser::format_source;
use yamark::plugins::PluginRegistry;

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static REALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static COUNT: Cell<bool> = const { Cell::new(false) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNT.with(Cell::get) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNT.with(Cell::get) {
            REALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, size) }
    }
}

fn format(input: &str) -> String {
    format_source(
        FileKind::Markdown,
        input.to_owned(),
        FormatOptions {
            markdown_wrap: MarkdownWrap::Sentence,
            skip_embedded_formatters: true,
            ..FormatOptions::default()
        },
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap()
}

#[test]
fn consecutive_blocks_keep_literal_and_preservation_state_separate() {
    let input = "First  [link](https://example.org/a) and `literal  spaces`.\n\nSecond  [site](https://example.org/b) has more words.\n\n<!-- fmt: skip -->\nKeep   spaces.  \t\n\nLast   `code  span` and [end](https://example.org/c).";
    let expected = "First [link](https://example.org/a) and `literal  spaces`.\n\nSecond [site](https://example.org/b) has more words.\n\n<!-- fmt: skip -->\nKeep   spaces.  \t\n\nLast `code  span` and [end](https://example.org/c).\n";
    assert_eq!(format(input), expected);
    assert_eq!(format(expected), expected);
}

#[test]
fn empty_and_mixed_line_ending_documents_keep_their_eof_policy() {
    assert_eq!(format(""), "");
    let input = "First  `code  span`.\r\n\r\nSecond  [link](https://example.org/a).\n\n<!-- fmt: skip -->\rKeep   mixed.  \t\r\n\r\nLast   prose.";
    let expected = "First `code  span`.\r\n\r\nSecond [link](https://example.org/a).\n\n<!-- fmt: skip -->\rKeep   mixed.  \t\r\n\r\nLast prose.\r\n";
    assert_eq!(format(input), expected);
    assert_eq!(format(expected), expected);
    let preserved_eof = "<!-- fmt: skip -->\nKeep   EOF.  \t";
    assert_eq!(format(preserved_eof), preserved_eof);
}

#[test]
fn sibling_block_allocation_count_stays_bounded() {
    let input = "A [link](https://example.org/a) and `code  span`.\n\n".repeat(1_000);
    let config = Config::default();
    let plugins = PluginRegistry::default();
    let options = FormatOptions {
        markdown_wrap: MarkdownWrap::Sentence,
        skip_embedded_formatters: true,
        ..FormatOptions::default()
    };
    ALLOCATIONS.store(0, Ordering::Relaxed);
    REALLOCATIONS.store(0, Ordering::Relaxed);
    COUNT.with(|count| count.set(true));
    let output = format_source(FileKind::Markdown, input, options, &config, &plugins).unwrap();
    COUNT.with(|count| count.set(false));
    assert_eq!(output.matches("[link]").count(), 1_000);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    let reallocations = REALLOCATIONS.load(Ordering::Relaxed);
    assert!(
        allocations <= 18_000,
        "{allocations} allocations and {reallocations} reallocations"
    );
}
