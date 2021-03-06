//! A File system warmer
//!
//! Cloud providers tent to restore volumes from snapshots in a cold state:
//!
//! > For volumes that were created from snapshots, the storage blocks must be pulled down from
//! Amazon S3 and written to the volume before you can access them. This preliminary action takes
//! time and can cause a significant increase in the latency of I/O operations the first time
//! each block is accessed ([source][ebs-initialize]).
//!
//! It has methods to estimates total size of particular folder and then recursively read files
//! in a thread pool.
//!
//! It implements `Iterator` giving an access to the warming process intermediate state.
//! Refer to [cli example] for progress bar implementation.
//!
//! [ebs-initialize]: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ebs-initialize.html
//! [cli example]: https://github.com/imbolc/warm-fs/blob/main/examples/cli.rs

#![warn(clippy::all, missing_docs, nonstandard_style, future_incompatible)]

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use threadpool::ThreadPool;
use walkdir::WalkDir;

/// The warmer
#[derive(Default)]
pub struct Warmer {
    dirs: Vec<PathBuf>,
    files: Vec<PathBuf>,
    num_threads: usize,
    follow_links: bool,
}

/// Iterator over the size estimation / file reading bytes
pub struct Iter {
    rx: Receiver<u64>,
}

impl Warmer {
    /// Creates a new warmer
    pub fn new(num_threads: usize, follow_links: bool) -> Self {
        Self {
            num_threads,
            follow_links,
            ..Default::default()
        }
    }

    /// Adds folders to walk recursively
    pub fn add_dirs(&mut self, paths: &[impl AsRef<Path>]) {
        let mut paths: Vec<_> = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
        self.dirs.append(&mut paths);
    }

    /// Adds files directly to avoid folders traversal
    pub fn add_files(&mut self, paths: &[impl AsRef<Path>]) {
        let mut paths: Vec<_> = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
        self.files.append(&mut paths);
    }

    /// Estimates total size to read, returns the total number of bytes
    pub fn estimate(&self) -> u64 {
        self.iter_estimate().sum()
    }

    /// Read files, returns the total number of read bytes
    pub fn warm(&self) -> u64 {
        self.iter_warm().sum()
    }

    /// Estimates total size to read, returns an iterator over file sizes
    pub fn iter_estimate(&self) -> Iter {
        let (tx, rx) = channel();
        let dirs = self.dirs.clone();
        let files = self.files.clone();
        let num_threads = self.num_threads;
        let follow_links = self.follow_links;
        std::thread::spawn(move || {
            let pool = ThreadPool::new(num_threads);
            for file in files {
                let tx = tx.clone();
                pool.execute(move || {
                    if let Ok(Some(file)) = resolve_file(file) {
                        if let Ok(size) = file.metadata().map(|m| m.len()) {
                            tx.send(size).ok();
                        }
                    }
                });
            }
            for dir in dirs {
                for entry in walker(dir, follow_links) {
                    let tx = tx.clone();
                    pool.execute(move || {
                        if let Ok(size) = entry.metadata().map(|m| m.len()) {
                            tx.send(size).ok();
                        }
                    });
                }
            }
        });
        Iter { rx }
    }

    /// Reads files, returns an iterator over the read number of bytes
    pub fn iter_warm(&self) -> Iter {
        let (tx, rx) = channel();
        let dirs = self.dirs.clone();
        let files = self.files.clone();
        let num_threads = self.num_threads;
        let follow_links = self.follow_links;
        std::thread::spawn(move || {
            let pool = ThreadPool::new(num_threads);
            for file in files {
                let tx = tx.clone();
                pool.execute(move || {
                    if let Ok(Some(file)) = resolve_file(file) {
                        warm_file(file, tx);
                    }
                });
            }
            for dir in dirs {
                for entry in walker(dir, follow_links) {
                    let tx = tx.clone();
                    pool.execute(move || warm_file(entry.path(), tx));
                }
            }
        });
        Iter { rx }
    }
}

/// Checks if it's a file and resolves a possible simlink
fn resolve_file(path: PathBuf) -> io::Result<Option<PathBuf>> {
    if path.is_file() {
        Ok(Some(path))
    } else if path.is_symlink() {
        path.canonicalize().map(Some)
    } else {
        Ok(None)
    }
}

/// Warms a file
fn warm_file(path: impl AsRef<Path>, tx: Sender<u64>) {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0; 1024];
        loop {
            let count = file.read(&mut buffer).unwrap_or_default();
            if count == 0 {
                break;
            }
            tx.send(count as u64).ok();
        }
    }
}

/// Initializes and returns a `walkdir::WalkDir` instance
fn walker(path: impl AsRef<Path>, follow_links: bool) -> impl Iterator<Item = walkdir::DirEntry> {
    let mut w = WalkDir::new(path);
    if follow_links {
        w = w.follow_links(true);
    }
    w.into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
}

impl Iterator for Iter {
    type Item = u64;

    /// Returns estimated / read number of bytes
    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}
