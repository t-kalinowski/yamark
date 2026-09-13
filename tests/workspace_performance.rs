use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use yamark::core::document::FormatOptions;
use yamark::workspace::{FormatMode, format_paths};

struct PeakAllocator;

static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static GLOBAL: PeakAllocator = PeakAllocator;

fn record_allocation(bytes: usize) {
    let live = LIVE_BYTES.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for PeakAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let ptr = unsafe { System.realloc(ptr, layout, size) };
        if !ptr.is_null() {
            if size >= layout.size() {
                record_allocation(size - layout.size());
            } else {
                LIVE_BYTES.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        ptr
    }
}

#[test]
fn check_mode_releases_formatted_text_before_collecting_results() {
    let dir = tempfile::tempdir().unwrap();
    let workers = std::thread::available_parallelism().map_or(1, |count| count.get());
    // Enough files per worker to distinguish retained results from active jobs.
    let files = 64 * workers;
    let input = format!("#   {}\n", "a".repeat(16_384));
    for index in 0..files {
        fs::write(dir.path().join(format!("{index}.md")), &input).unwrap();
    }
    let baseline = LIVE_BYTES.load(Ordering::Relaxed);
    PEAK_BYTES.store(baseline, Ordering::Relaxed);
    let run = format_paths(
        vec![dir.path().to_owned()],
        FormatOptions::default(),
        FormatMode::Check,
        None,
    )
    .unwrap();
    let peak = PEAK_BYTES.load(Ordering::Relaxed) - baseline;
    assert_eq!(run.summary.formatted, files);
    assert_eq!(run.summary.failed, 0);
    assert!(run.diffs.is_empty());
    assert_eq!(fs::read_to_string(dir.path().join("0.md")).unwrap(), input);
    assert!(
        peak < files * input.len() / 2,
        "check mode used {peak} extra live bytes for {files} files"
    );
}
