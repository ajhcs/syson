// SPDX-License-Identifier: EPL-2.0
//! One instrument included in separate library/integration test executables.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub requested: usize,
    pub largest: usize,
}
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static STATS: Cell<Stats> = const { Cell::new(Stats { requested: 0, largest: 0 }) };
}
fn record(size: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = STATS.try_with(|value| {
            let mut stats = value.get();
            stats.requested = stats.requested.saturating_add(size);
            stats.largest = stats.largest.max(size);
            value.set(stats);
        });
    }
}
struct Observed;
// SAFETY: unchanged pointers/layouts go to System. Constant TLS is nonallocating.
// Only successful requests count. Realloc charges its full new request; these
// are aggregate requested bytes, not retained/live memory or process RSS.
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            record(size);
        }
        next
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: Observed = Observed;
pub fn observed<T>(operation: impl FnOnce() -> T) -> (T, Stats) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TRACK.with(|value| value.set(false));
        }
    }
    STATS.with(|value| value.set(Stats::default()));
    TRACK.with(|value| value.set(true));
    let reset = Reset;
    let result = operation();
    drop(reset);
    (result, STATS.with(Cell::get))
}
