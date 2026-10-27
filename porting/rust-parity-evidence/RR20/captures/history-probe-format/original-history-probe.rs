// SPDX-License-Identifier: EPL-2.0
//! External host-only measurement consumer; never compiled inside model's
//! forbid(unsafe_code) library. Root owns its separate Cargo manifest/execution.
use std::{alloc::{GlobalAlloc, Layout, System}, cell::Cell, time::Instant};
use syson_model::{Document, Editor};
#[derive(Clone, Copy, Default)]
struct Counts { live: i64, peak: i64, requested: u64, largest: usize, calls: u64 }
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static COUNTS: Cell<Counts> = const { Cell::new(Counts {live:0,peak:0,requested:0,largest:0,calls:0}) };
}
fn record(delta: i64, requested: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = COUNTS.try_with(|cell| { let mut c=cell.get(); c.live+=delta; c.peak=c.peak.max(c.live); c.requested+=requested as u64; c.largest=c.largest.max(requested); c.calls+=u64::from(requested!=0); cell.set(c); });
    }
}
struct Observed;
// SAFETY: Original allocator pointers/layouts reach System unchanged. Constant
// TLS bookkeeping owns no allocation metadata and tolerates thread teardown.
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {let p=unsafe{System.alloc(l)};if !p.is_null(){record(l.size() as i64,l.size());}p}
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 {let p=unsafe{System.alloc_zeroed(l)};if !p.is_null(){record(l.size() as i64,l.size());}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){record(-(l.size() as i64),0);unsafe{System.dealloc(p,l)};}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {let q=unsafe{System.realloc(p,l,n)};if !q.is_null(){record(n as i64-l.size() as i64,n);}q}
}
#[global_allocator]
static ALLOCATOR: Observed=Observed;
fn fixture(nodes:usize)->Document {
    let content:Vec<_>=(0..nodes).map(|n|serde_json::json!({"id":format!("10000000-0000-4000-8000-{n:012x}"),"eClass":"sysml:Package","data":{"elementId":format!("20000000-0000-4000-8000-{n:012x}"),"declaredName":format!("Package{n}")}})).collect();
    Document::from_json(&serde_json::json!({"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":content}).to_string()).unwrap()
}
fn rss()->serde_json::Value {
    // This is a separate Linux process observation, not an allocator oracle.
    let status=std::fs::read_to_string("/proc/self/status").unwrap();
    let field=|name:&str| status.lines().find(|line|line.starts_with(name)).unwrap().split_whitespace().nth(1).unwrap().parse::<u64>().unwrap();
    serde_json::json!({"vm_rss_kib":field("VmRSS:"),"vm_hwm_kib":field("VmHWM:")})
}
fn main(){
    let args:Vec<_>=std::env::args().collect();assert_eq!(args.len(),3);
    let nodes:usize=args[1].parse().unwrap();let commits:usize=args[2].parse().unwrap();
    assert!([1000,4096].contains(&nodes)&&[64,256].contains(&commits));
    let fixture=fixture(nodes);let initial_rss=rss();let id="20000000-0000-4000-8000-000000000000";
    COUNTS.set(Counts::default());TRACK.set(true);
    // All retained Editor storage is allocated inside observation. The fixture
    // stays outside and alive until after tracking ends, avoiding live skew.
    let mut editor=Editor::new(fixture.clone());let initial=COUNTS.get();
    let mut begin_bytes=0;let mut apply_bytes=0;let mut commit_bytes=0;let started=Instant::now();
    for n in 1..=commits {
        let name=format!("Revision{n}");let before=COUNTS.get().requested;
        let mut tx=editor.begin();begin_bytes+=COUNTS.get().requested-before;let before=COUNTS.get().requested;
        tx.rename(id,&name).unwrap();apply_bytes+=COUNTS.get().requested-before;let before=COUNTS.get().requested;
        assert!(editor.commit(tx).unwrap());commit_bytes+=COUNTS.get().requested-before;
    }
    let elapsed=started.elapsed().as_micros();let retained=COUNTS.get();TRACK.set(false);let retained_rss=rss();
    let before_name=editor.document().element(id).unwrap()["data"]["declaredName"].as_str().unwrap().to_owned();
    TRACK.set(true);let before_cancel=COUNTS.get();let started=Instant::now();let mut cancelled=editor.begin();cancelled.rename(id,"Cancelled").unwrap();cancelled.cancel();let cancel_us=started.elapsed().as_micros();let after_cancel=COUNTS.get();TRACK.set(false);
    assert_eq!(editor.document().element(id).unwrap()["data"]["declaredName"].as_str(),Some(before_name.as_str()));assert_eq!(after_cancel.live,before_cancel.live);
    TRACK.set(true);let before=COUNTS.get().requested;let mut undos=0;let started=Instant::now();while editor.undo(){undos+=1;}let undo_us=started.elapsed().as_micros();let undo_requested=COUNTS.get().requested-before;let before=COUNTS.get().requested;let mut redos=0;let started=Instant::now();while editor.redo(){redos+=1;}let redo_us=started.elapsed().as_micros();let redo_requested=COUNTS.get().requested-before;TRACK.set(false);
    assert_eq!(undos,redos);assert_eq!(editor.document().element(id).unwrap()["data"]["declaredName"].as_str(),Some(before_name.as_str()));
    println!("{}",serde_json::json!({"scope":"host-only main-thread requested allocations, not allocator overhead or exact copied bytes","nodes":nodes,"commits":commits,"initial_live":initial.live,"retained_live":retained.live,"history_live_delta":retained.live-initial.live,"peak_live":retained.peak,"largest_request":retained.largest,"allocation_calls":retained.calls,"requested_begin_bytes":begin_bytes,"requested_apply_bytes":apply_bytes,"requested_commit_bytes":commit_bytes,"edit_total_us":elapsed,"cancel_us":cancel_us,"cancel_requested_bytes":after_cancel.requested-before_cancel.requested,"cancel_live_delta":after_cancel.live-before_cancel.live,"undo_steps":undos,"redo_steps":redos,"undo_us":undo_us,"redo_us":redo_us,"undo_requested_bytes":undo_requested,"redo_requested_bytes":redo_requested,"rss_after_fixture":initial_rss,"rss_after_edits":retained_rss}));
}
