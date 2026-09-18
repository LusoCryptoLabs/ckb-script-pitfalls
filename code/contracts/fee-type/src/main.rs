//! `fee-type`: spending a cell of this type owes a fee.
//!
//! args = `treasury_lock_hash (32) ‖ fee (8, little-endian shannons)`. Creating a cell
//! of this type is free; consuming one requires the outputs to pay `fee` to the
//! treasury. That is the whole contract, and it is enough to exhibit two pitfalls:
//!
//!   * **01, a payment sum is safe only against itself.** As first written, each
//!     instance requires its own `fee` and reads the whole transaction's outputs. Two
//!     cells with different fees are two script groups; both read the same outputs, so
//!     one output covering the larger fee satisfies both. Fixed: every instance sums
//!     the fee of EVERY input carrying this code, each at its own args, and demands
//!     that total, which cannot be split.
//!   * **02, a typed output meets the capacity.** As first written, any output under the
//!     treasury lock counts, whatever type script it carries; a hostile type meets the
//!     capacity and the treasury cannot spend the cell. Fixed: only pure outputs count.
//!
//! Build without features for the pitfall, with `--features fixed` for the correction.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed::Script,
    error::SysError,
    high_level::{load_cell_capacity, load_cell_lock_hash, load_script},
};
#[cfg(feature = "fixed")]
use ckb_std::{
    ckb_types::prelude::Entity,
    high_level::{load_cell_type, load_cell_type_hash},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

const ARGS_LEN: usize = 40;

#[repr(i8)]
enum Err {
    Encoding = 1,
    BadArgs = 2,
    /// The outputs do not pay what is owed.
    Unpaid = 3,
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(e) => e as i8,
    }
}

fn run() -> Result<(), Err> {
    let script = load_script().map_err(|_| Err::Encoding)?;
    let args = script.args().raw_data();
    if args.len() != ARGS_LEN {
        return Err(Err::BadArgs);
    }
    let treasury: [u8; 32] = args[..32].try_into().map_err(|_| Err::BadArgs)?;
    let fee = u64::from_le_bytes(args[32..40].try_into().map_err(|_| Err::BadArgs)?);

    // Creation owes nothing: only spending a cell of this type does.
    if !any_group_input()? {
        return Ok(());
    }
    if paid_to(&treasury)? < owed(&script, fee)? {
        return Err(Err::Unpaid);
    }
    Ok(())
}

fn any_group_input() -> Result<bool, Err> {
    match load_cell_capacity(0, Source::GroupInput) {
        Ok(_) => Ok(true),
        Err(SysError::IndexOutOfBound) => Ok(false),
        Err(_) => Err(Err::Encoding),
    }
}

/// What this transaction owes the treasury.
///
/// The pitfall: this instance's fee, and nothing else. Every instance in the transaction
/// reads the same outputs, so with two instances at fees 100 and 50, one output of 100
/// satisfies both: each sees enough for itself and nobody sees the sum.
#[cfg(not(feature = "fixed"))]
fn owed(_script: &Script, fee: u64) -> Result<u64, Err> {
    Ok(fee)
}

/// The fix: sum the fee of EVERY input carrying this code, each at its own args. Every
/// instance walks the same inputs, so every instance demands the same indivisible total.
#[cfg(feature = "fixed")]
fn owed(script: &Script, _fee: u64) -> Result<u64, Err> {
    let code = script.code_hash();
    let htype = script.hash_type();
    let mut total: u64 = 0;
    let mut i = 0usize;
    loop {
        match load_cell_type(i, Source::Input) {
            Ok(Some(t)) => {
                if t.code_hash().as_slice() == code.as_slice() && t.hash_type().as_slice() == htype.as_slice() {
                    let a = t.args().raw_data();
                    if a.len() == ARGS_LEN {
                        let f = u64::from_le_bytes(a[32..40].try_into().map_err(|_| Err::BadArgs)?);
                        total = total.saturating_add(f);
                    }
                }
                i += 1;
            }
            Ok(None) => i += 1,
            Err(SysError::IndexOutOfBound) => return Ok(total),
            Err(_) => return Err(Err::Encoding),
        }
    }
}

/// What the outputs pay to `lock`.
fn paid_to(lock: &[u8; 32]) -> Result<u64, Err> {
    let mut paid: u64 = 0;
    let mut i = 0usize;
    loop {
        match load_cell_lock_hash(i, Source::Output) {
            Ok(h) => {
                if &h == lock && counts(i)? {
                    let c = load_cell_capacity(i, Source::Output).map_err(|_| Err::Encoding)?;
                    paid = paid.saturating_add(c);
                }
                i += 1;
            }
            Err(SysError::IndexOutOfBound) => return Ok(paid),
            Err(_) => return Err(Err::Encoding),
        }
    }
}

/// Does output `i` count as a payment? The pitfall: yes, whatever type script it carries.
#[cfg(not(feature = "fixed"))]
fn counts(_i: usize) -> Result<bool, Err> {
    Ok(true)
}

/// The fix: only a PURE output counts. A type the payee does not control meets the
/// capacity while the funds are not theirs to spend.
#[cfg(feature = "fixed")]
fn counts(i: usize) -> Result<bool, Err> {
    Ok(load_cell_type_hash(i, Source::Output).map_err(|_| Err::Encoding)?.is_none())
}
