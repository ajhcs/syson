use serde_json::json;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use syson_workflow::{FsSnapshotIo, SnapshotIo};

pub const BARRIERS: [&str; 4] = ["sync_staged", "publish", "sync_directory", "remove_staged"];

#[derive(Default)]
pub struct Observation<'a> {
    pub barrier: Option<&'a str>,
    pub attempted: Vec<&'static str>,
    pub completed: Vec<&'static str>,
}

impl Observation<'_> {
    fn step<T>(
        &mut self,
        step: &'static str,
        operation: impl FnOnce(&mut FsSnapshotIo) -> io::Result<T>,
    ) -> io::Result<T> {
        self.attempted.push(step);
        let result = operation(&mut FsSnapshotIo)?;
        self.completed.push(step);
        if self.barrier == Some(step) {
            {
                let mut stderr = io::stderr().lock();
                writeln!(
                    stderr,
                    "{}",
                    json!({
                        "pid":std::process::id(), "barrier":step,
                        "attempted":self.attempted, "completed":self.completed
                    })
                )?;
                stderr.flush()?;
            }
            // No next save operation may begin after this marker. If the
            // supervisor exits, the pipe closes and the experiment refuses.
            io::stdin().lock().read_exact(&mut [0])?;
            return Err(io::Error::other("experiment barrier received input"));
        }
        Ok(result)
    }
}

impl SnapshotIo for Observation<'_> {
    fn create_staged(&mut self, path: &Path) -> io::Result<File> {
        self.step("create_staged", |io| io.create_staged(path))
    }
    fn write_staged(&mut self, file: &mut File, data: &[u8]) -> io::Result<()> {
        self.step("write_staged", |io| io.write_staged(file, data))
    }
    fn sync_staged(&mut self, file: &File) -> io::Result<()> {
        self.step("sync_staged", |io| io.sync_staged(file))
    }
    fn publish(&mut self, staged: &Path, destination: &Path) -> io::Result<()> {
        self.step("publish", |io| io.publish(staged, destination))
    }
    fn sync_directory(&mut self, directory: &Path) -> io::Result<()> {
        self.step("sync_directory", |io| io.sync_directory(directory))
    }
    fn remove_staged(&mut self, staged: &Path) -> io::Result<()> {
        self.step("remove_staged", |io| io.remove_staged(staged))
    }
}
