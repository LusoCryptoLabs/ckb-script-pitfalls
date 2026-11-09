//! `payout-lock`: a lock that releases a cell when a named recipient gets an exact amount.
//!
//! args = `batch_id (32) ‖ recipient_lock_hash (32) ‖ capacity (8, little-endian shannons)`.
//! The `batch_id` is what a channel or batch commitment does: it makes two otherwise
//! identical payouts two different lock scripts, hence two script groups. Spending a cell
//! under this lock requires the outputs to contain EXACTLY ONE output paying `recipient`
//! exactly `capacity`.
//!
//! It exhibits pitfall 01 in its strictest form. "Exactly one" looks like it cannot be
//! double counted, yet two instances (two batch ids, so two groups) owing the same
//! recipient the same amount are each answered by the same single output, and the
//! recipient is paid once. The rule even forces that shape: paying the recipient twice
//! makes the count two and is refused, so the honest form is the one the lock rejects.
//!
//! Build without features for the pitfall, with `--features fixed` for the correction.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_capacity, load_cell_lock_hash, load_script},
};
#[cfg(feature = "fixed")]
use ckb_std::{ckb_types::prelude::Entity, high_level::load_input};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

const ARGS_LEN: usize = 72;

#[repr(i8)]
enum Err {
    Encoding = 1,
    BadArgs = 2,
    /// The outputs do not pay the recipient exactly once.
    NotPaid = 3,
    /// The spent cell is not the transaction's first input (fix only).
    NotLeading = 4,
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
    let recipient: [u8; 32] = args[32..64].try_into().map_err(|_| Err::BadArgs)?;
    let capacity = u64::from_le_bytes(args[64..72].try_into().map_err(|_| Err::BadArgs)?);

    reserve_leading_input()?;
    if exact_payouts(&recipient, capacity)? != 1 {
        return Err(Err::NotPaid);
    }
    Ok(())
}

/// How many outputs pay `recipient` exactly `capacity`.
fn exact_payouts(recipient: &[u8; 32], capacity: u64) -> Result<u32, Err> {
    let mut matches = 0u32;
    let mut i = 0usize;
    loop {
        match load_cell_lock_hash(i, Source::Output) {
            Ok(h) => {
                if &h == recipient
                    && load_cell_capacity(i, Source::Output).map_err(|_| Err::Encoding)? == capacity
                {
                    matches += 1;
                }
                i += 1;
            }
            Err(SysError::IndexOutOfBound) => return Ok(matches),
            Err(_) => return Err(Err::Encoding),
        }
    }
}

/// The pitfall: nothing reserves the inputs, so two payout-lock cells (two batch ids, two
/// groups) coexist in one transaction and each is answered by the same single output.
#[cfg(not(feature = "fixed"))]
fn reserve_leading_input() -> Result<(), Err> {
    Ok(())
}

/// The fix: this cell must be the transaction's first input. Only one cell can be input 0,
/// so two instances cannot share a transaction and the output answers only one of them.
/// The trade is that two of these can no longer be batched in one transaction.
#[cfg(feature = "fixed")]
fn reserve_leading_input() -> Result<(), Err> {
    let group = load_input(0, Source::GroupInput).map_err(|_| Err::Encoding)?;
    let first = load_input(0, Source::Input).map_err(|_| Err::Encoding)?;
    if group.as_slice() != first.as_slice() {
        return Err(Err::NotLeading);
    }
    Ok(())
}
