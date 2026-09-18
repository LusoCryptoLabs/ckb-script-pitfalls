//! `owned-type`: a cell whose data names its owner, under an always-success lock.
//!
//! data = `owner (20 bytes, the first twenty of the owner's lock hash) ‖ value (8, LE)`.
//! The cell's own lock is always-success by design: all authorization lives here, and an
//! owner action is proven by spending any cell under the owner's lock.
//!
//! Three shapes, told apart by the group's arity and `witnesses[0].input_type`:
//!   * **create**: no input of the type. The owner must be usable: not all-zero, and
//!     not the cell's own always-success lock, which anybody can satisfy.
//!   * **bump**: one in, one out, permissionless. Anyone may raise `value`; nothing
//!     else in the data may change.
//!   * **edit**: one in, one out, owner-signed. Anything may change.
//!
//! Two pitfalls live here:
//!   * **03, a truncated compare fails open.** As first written, the create guard
//!     compares the twenty-byte owner with the thirty-two-byte cell lock hash. They
//!     are never equal, so the guard never fires, and a cell created with its owner set
//!     to the always-success prefix is public property: `edit` is satisfied by the
//!     cell's own input. Fixed: compare the prefix.
//!   * **07, capacity is unguarded on permissionless actions.** `bump` pins every byte
//!     of data and, as first written, nothing about the capacity, so a stranger bumps
//!     the value and takes the owner's rent. Fixed: the output may not hold less.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_capacity, load_cell_data, load_cell_lock_hash, load_witness_args},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

const OWNER_LEN: usize = 20;
const DATA_LEN: usize = OWNER_LEN + 8;

#[repr(i8)]
#[allow(dead_code)] // CapacityShrunk exists only in the fixed build
enum Err {
    Encoding = 1,
    BadData = 2,
    NullOwner = 3,
    OwnerIsCellLock = 4,
    /// The surviving cell would hold less capacity than it did.
    CapacityShrunk = 5,
    Arity = 6,
    UnknownAction = 7,
    Unauthorized = 8,
    /// A bump that did not raise the value, or changed the owner.
    Drift = 9,
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(e) => e as i8,
    }
}

fn count(source: Source) -> Result<usize, Err> {
    let mut i = 0usize;
    loop {
        match load_cell_capacity(i, source) {
            Ok(_) => i += 1,
            Err(SysError::IndexOutOfBound) => return Ok(i),
            Err(_) => return Err(Err::Encoding),
        }
    }
}

fn run() -> Result<(), Err> {
    let outs = count(Source::GroupOutput)?;
    let ins = count(Source::GroupInput)?;

    // Every output of the type must be well formed and have a usable owner.
    for i in 0..outs {
        let d = load_cell_data(i, Source::GroupOutput).map_err(|_| Err::Encoding)?;
        if d.len() != DATA_LEN {
            return Err(Err::BadData);
        }
        let owner = &d[..OWNER_LEN];
        if owner.iter().all(|&b| b == 0) {
            return Err(Err::NullOwner);
        }
        let lock = load_cell_lock_hash(i, Source::GroupOutput).map_err(|_| Err::Encoding)?;
        if owner_is_cell_lock(owner, &lock) {
            return Err(Err::OwnerIsCellLock);
        }
    }

    if ins == 0 {
        return Ok(()); // create
    }
    if ins != 1 || outs != 1 {
        return Err(Err::Arity);
    }
    let din = load_cell_data(0, Source::GroupInput).map_err(|_| Err::Encoding)?;
    let dout = load_cell_data(0, Source::GroupOutput).map_err(|_| Err::Encoding)?;
    if din.len() != DATA_LEN {
        return Err(Err::BadData);
    }
    let wa = load_witness_args(0, Source::Input).map_err(|_| Err::UnknownAction)?;
    let action = wa.input_type().to_opt().ok_or(Err::UnknownAction)?.raw_data();
    match action.as_ref() {
        b"bump" => {
            if din[..OWNER_LEN] != dout[..OWNER_LEN] {
                return Err(Err::Drift);
            }
            let vin = u64::from_le_bytes(din[OWNER_LEN..].try_into().map_err(|_| Err::BadData)?);
            let vout = u64::from_le_bytes(dout[OWNER_LEN..].try_into().map_err(|_| Err::BadData)?);
            if vout <= vin {
                return Err(Err::Drift);
            }
            capacity_ok()
        }
        b"edit" => require_owner(&din[..OWNER_LEN]),
        _ => Err(Err::UnknownAction),
    }
}

/// The pitfall: twenty bytes against thirty-two. Never equal, so never refused.
#[cfg(not(feature = "fixed"))]
fn owner_is_cell_lock(owner: &[u8], cell_lock: &[u8; 32]) -> bool {
    owner == &cell_lock[..]
}

/// The fix: compare what is stored with the same length of what it was cut from.
#[cfg(feature = "fixed")]
fn owner_is_cell_lock(owner: &[u8], cell_lock: &[u8; 32]) -> bool {
    owner == &cell_lock[..OWNER_LEN]
}

/// The pitfall: the data is pinned, the money is not.
#[cfg(not(feature = "fixed"))]
fn capacity_ok() -> Result<(), Err> {
    Ok(())
}

/// The fix: the surviving cell may not hold less than it did.
#[cfg(feature = "fixed")]
fn capacity_ok() -> Result<(), Err> {
    let cin = load_cell_capacity(0, Source::GroupInput).map_err(|_| Err::Encoding)?;
    let cout = load_cell_capacity(0, Source::GroupOutput).map_err(|_| Err::Encoding)?;
    if cout < cin {
        return Err(Err::CapacityShrunk);
    }
    Ok(())
}

/// Proof of ownership: some input sits under a lock whose hash starts with `owner`.
fn require_owner(owner: &[u8]) -> Result<(), Err> {
    let mut i = 0usize;
    loop {
        match load_cell_lock_hash(i, Source::Input) {
            Ok(h) => {
                if h[..OWNER_LEN] == *owner {
                    return Ok(());
                }
                i += 1;
            }
            Err(SysError::IndexOutOfBound) => return Err(Err::Unauthorized),
            Err(_) => return Err(Err::Encoding),
        }
    }
}
