//! `timed-lock`: a cell that may be spent once a moment has passed.
//!
//! args = `unlock_at (8, little-endian unix seconds)`. The proof is the input's `since`:
//! consensus refuses to include the transaction until the condition it encodes holds,
//! so the script needs no clock and no header. What the script must check is that the
//! `since` encodes the condition it thinks it does.
//!
//! **09, `since` flags.** The top byte of `since` carries the flags: bit 63 relative,
//! bits 62 and 61 the metric (`10` is timestamp). As first written the script compared
//! only the low 56 bits with `unlock_at`, so a RELATIVE timestamp of `unlock_at` seconds
//! (top byte 0xC0) passed the check, and consensus then read it as "this input is at
//! least `unlock_at` seconds old", a completely different condition that a fresh cell
//! meets after a few days rather than at the intended date. Fixed: the top byte must be
//! exactly 0x40, absolute timestamp, before the value means anything.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    high_level::{load_input, load_script},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

// Both are used only by the fixed build; the vulnerable one never reads the flags.
#[allow(dead_code)]
const ABS_TIMESTAMP_FLAG: u64 = 0x40;

#[repr(i8)]
#[allow(dead_code)]
enum Err {
    Encoding = 1,
    TooEarly = 2,
    /// The since is not an absolute timestamp, so its value proves nothing about the date.
    WrongFlag = 3,
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
    if args.len() != 8 {
        return Err(Err::Encoding);
    }
    let unlock_at = u64::from_le_bytes(args[..8].try_into().map_err(|_| Err::Encoding)?);
    let input = load_input(0, Source::GroupInput).map_err(|_| Err::Encoding)?;
    let since: u64 = input.since().unpack();
    check_flags(since)?;
    if (since & 0x00FF_FFFF_FFFF_FFFF) < unlock_at {
        return Err(Err::TooEarly);
    }
    Ok(())
}

/// The pitfall: the flags are never looked at.
#[cfg(not(feature = "fixed"))]
fn check_flags(_since: u64) -> Result<(), Err> {
    Ok(())
}

/// The fix: an absolute timestamp, and nothing else, before the value is read.
#[cfg(feature = "fixed")]
fn check_flags(since: u64) -> Result<(), Err> {
    if (since >> 56) != ABS_TIMESTAMP_FLAG {
        return Err(Err::WrongFlag);
    }
    Ok(())
}
