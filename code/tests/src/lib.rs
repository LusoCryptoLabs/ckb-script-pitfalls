//! Test support: load one of the two builds of a toy script.
//!
//! `make build` leaves the scripts as first written under `target/vulnerable/` and with
//! the correction under `target/fixed/`. A test names which one it runs, so it can say
//! "accepted here, refused there" about the same transaction.
use ckb_testtool::ckb_types::bytes::Bytes;
use std::{env, fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Build {
    /// The script as it was first written: the pitfall.
    Vulnerable,
    /// The script with the correction.
    Fixed,
}

pub fn load(build: Build, name: &str) -> Bytes {
    let root = env::var("PITFALLS_TARGET").unwrap_or_else(|_| "../target".to_string());
    let mut path = PathBuf::from(root);
    path.push(match build {
        Build::Vulnerable => "vulnerable",
        Build::Fixed => "fixed",
    });
    path.push(name);
    Bytes::from(fs::read(&path).unwrap_or_else(|e| {
        panic!("failed to read {path:?}: {e}\nbuild the scripts first: `make build` (from code/)")
    }))
}
