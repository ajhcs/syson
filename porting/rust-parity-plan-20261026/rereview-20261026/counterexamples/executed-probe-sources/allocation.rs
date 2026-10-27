// Review-only measurement shim; never linked into production modules.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering::Relaxed};
struct Tracked;
#[global_allocator]
static ALLOCATOR: Tracked=Tracked;
static LIVE: AtomicUsize=AtomicUsize::new(0);
static PEAK: AtomicUsize=AtomicUsize::new(0);
static MAX_ONE: AtomicUsize=AtomicUsize::new(0);
static ENABLED: AtomicBool=AtomicBool::new(false);
fn add(bytes: usize) {
    let live=LIVE.fetch_add(bytes,Relaxed)+bytes;
    if ENABLED.load(Relaxed) { PEAK.fetch_max(live,Relaxed); MAX_ONE.fetch_max(bytes,Relaxed); }
}
unsafe impl GlobalAlloc for Tracked {
    unsafe fn alloc(&self,layout:Layout)->*mut u8 {
        let p=unsafe {System.alloc(layout)};if !p.is_null(){add(layout.size());}p
    }
    unsafe fn alloc_zeroed(&self,layout:Layout)->*mut u8 {
        let p=unsafe {System.alloc_zeroed(layout)};if !p.is_null(){add(layout.size());}p
    }
    unsafe fn dealloc(&self,p:*mut u8,layout:Layout) {
        LIVE.fetch_sub(layout.size(),Relaxed);unsafe {System.dealloc(p,layout)}
    }
    unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8 {
        let q=unsafe {System.realloc(p,layout,size)};
        if !q.is_null() { LIVE.fetch_sub(layout.size(),Relaxed);add(size); }q
    }
}
pub fn begin()->usize {
    let base=LIVE.load(Relaxed);PEAK.store(base,Relaxed);MAX_ONE.store(0,Relaxed);ENABLED.store(true,Relaxed);base
}
pub fn end(base:usize)->(usize,usize) {
    ENABLED.store(false,Relaxed);(PEAK.load(Relaxed).saturating_sub(base),MAX_ONE.load(Relaxed))
}
