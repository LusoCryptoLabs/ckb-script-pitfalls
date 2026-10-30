//! `allowance-lock`: a cell that anyone may spend, as long as its owner gets back what it holds,
//! less an allowance the cell names (the fee a relayer may keep for the work).
//!
//! args = `owner lock hash (32) · allowance (8, little-endian shannons)`. Cells of one owner are
//! counted together across the whole transaction, as entry 01 asks: what all of the owner's
//! cells hold, less what all of them allow, must reach the owner's plain outputs.
//!
//! **16, an allowance anyone can claim.** The allowances are summed as each cell states them,
//! and anyone may make a cell under this lock naming any owner. So a stranger makes one for
//! the victim that allows more than it holds, spends it beside the victim's real cell, and the
//! sum of allowances covers both: the victim's CKB goes where the stranger likes. Fixed: a
//! cell's allowance counts up to the CKB that cell holds, so one made by anyone adds nothing
//! the owner could lose.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_capacity, load_cell_lock, load_cell_lock_hash, load_cell_type, load_script},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

#[repr(i8)]
enum Err {
    Encoding = 1,
    /// The owner's plain outputs and the allowances do not reach what the owner's cells held.
    Underpaid = 2,
}

impl From<SysError> for Err {
    fn from(_: SysError) -> Self {
        Err::Encoding
    }
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(e) => e as i8,
    }
}

fn terms(args: &[u8]) -> Option<([u8; 32], u64)> {
    if args.len() != 40 {
        return None;
    }
    Some((args[..32].try_into().ok()?, u64::from_le_bytes(args[32..].try_into().ok()?)))
}

fn run() -> Result<(), Err> {
    let me = load_script()?;
    let (owner, _) = terms(&me.args().raw_data()).ok_or(Err::Encoding)?;

    // Every input under this lock code for this owner, whatever its allowance.
    let mut held: u128 = 0;
    let mut allowed: u128 = 0;
    let mut i = 0;
    loop {
        let lock = match load_cell_lock(i, Source::Input) {
            Ok(l) => l,
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(e.into()),
        };
        if lock.code_hash().raw_data() == me.code_hash().raw_data() && lock.hash_type() == me.hash_type() {
            if let Some((o, allowance)) = terms(&lock.args().raw_data()) {
                if o == owner {
                    let capacity = load_cell_capacity(i, Source::Input)?;
                    held += u128::from(capacity);
                    allowed += u128::from(allowance_of(allowance, capacity));
                }
            }
        }
        i += 1;
    }

    // What the owner gets: plain cells under their lock (entry 02 is not the subject here).
    let mut paid: u128 = 0;
    let mut k = 0;
    loop {
        let lock = match load_cell_lock_hash(k, Source::Output) {
            Ok(l) => l,
            Err(SysError::IndexOutOfBound) => break,
            Err(e) => return Err(e.into()),
        };
        if lock == owner && load_cell_type(k, Source::Output)?.is_none() {
            paid += u128::from(load_cell_capacity(k, Source::Output)?);
        }
        k += 1;
    }
    if paid + allowed < held {
        return Err(Err::Underpaid);
    }
    Ok(())
}

/// The pitfall: an allowance counts as the cell states it.
#[cfg(not(feature = "fixed"))]
fn allowance_of(allowance: u64, _capacity: u64) -> u64 {
    allowance
}

/// The fix: never more than the cell itself holds.
#[cfg(feature = "fixed")]
fn allowance_of(allowance: u64, capacity: u64) -> u64 {
    allowance.min(capacity)
}
