use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use yamark::config::Config;
use yamark::core::document::{FileKind, FormatOptions};
use yamark::core::parser::format_source;
use yamark::plugins::PluginRegistry;

struct PeakAllocator;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static COUNT: Cell<bool> = const { Cell::new(false) };
}

#[global_allocator]
static ALLOCATOR: PeakAllocator = PeakAllocator;

fn counting() -> bool {
    COUNT.with(Cell::get)
}

unsafe impl GlobalAlloc for PeakAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && counting() {
            let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(live, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        if counting() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, layout, size) };
        if !result.is_null() && counting() {
            if size >= layout.size() {
                let live =
                    LIVE.fetch_add(size - layout.size(), Ordering::Relaxed) + size - layout.size();
                PEAK.fetch_max(live, Ordering::Relaxed);
            } else {
                LIVE.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        result
    }
}

#[test]
fn finalized_markdown_releases_planning_storage_before_emission() {
    let unit = "Brief prose with [a useful link](https://example.org/guide/section?id=42) and a few words.\n\n";
    COUNT.with(|count| count.set(true));
    let input = unit.repeat(2_800);
    let output = format_source(
        FileKind::Markdown,
        input,
        FormatOptions::default(),
        &Config::default(),
        &PluginRegistry::default(),
    )
    .unwrap();
    let peak = PEAK.load(Ordering::Relaxed);
    COUNT.with(|count| count.set(false));
    assert_eq!(output.matches("[a useful link]").count(), 2_800);
    assert!(peak < 3_500_000, "peak live heap: {peak} bytes");
}
