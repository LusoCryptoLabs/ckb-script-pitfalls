//! `epoch-lock`: a cell that may be spent once an epoch has passed, where the boundary is
//! an epoch with a fraction (number + index/length).
//!
//! args = `boundary (8, little-endian)`, an absolute EpochNumberWithFraction `since` value
//! (top byte 0x20): the earliest epoch, as number + index/length, at which the cell opens.
//! The proof is the input's `since`, also an absolute epoch; consensus refuses to include
//! the transaction until that epoch has passed, so the script compares the two fractions.
//!
//! **17, an epoch `since` fraction compares unnormalized.** An EpochNumberWithFraction is
//! number + index/length, and a well formed fraction has `length >= 1` and `index < length`.
//! As first written the script compares the two by cross multiplying,
//! `s.index * b.length >= b.index * s.length`, without checking that the lengths are real. A
//! `since` of `number + 0/0` (index 0, length 0) makes both sides zero, so `0 >= 0` holds and
//! `number + 0/0` is accepted as if it were `>= number + index/length`: the cell opens at the
//! start of the epoch instead of part way through. Fixed: a fraction with `length == 0`, or
//! `index >= length`, is refused before it is compared.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    high_level::{load_input, load_script},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

/// Absolute epoch: bit 63 clear (absolute), metric bits `01` (epoch). Top byte 0x20.
const ABS_EPOCH_FLAG: u64 = 0x20;

#[repr(i8)]
#[allow(dead_code)]
enum Err {
    Encoding = 1,
    TooEarly = 2,
    /// The since fraction is not well formed, so its value proves nothing. Fixed build only.
    BadSince = 3,
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(e) => e as i8,
    }
}

/// number (bits 0..24), index (24..40), length (40..56).
fn decode(v: u64) -> (u64, u64, u64) {
    (v & 0x00FF_FFFF, (v >> 24) & 0xFFFF, (v >> 40) & 0xFFFF)
}

fn run() -> Result<(), Err> {
    let args = load_script().map_err(|_| Err::Encoding)?.args().raw_data();
    if args.len() != 8 {
        return Err(Err::Encoding);
    }
    let boundary = u64::from_le_bytes(args[..8].try_into().map_err(|_| Err::Encoding)?);
    let since: u64 = load_input(0, Source::GroupInput).map_err(|_| Err::Encoding)?.since().unpack();
    // Both must be absolute epochs. The metric and relative bit are pitfall 09, not this one.
    if (since >> 56) != ABS_EPOCH_FLAG || (boundary >> 56) != ABS_EPOCH_FLAG {
        return Err(Err::Encoding);
    }
    let (sn, si, sl) = decode(since);
    let (bn, bi, bl) = decode(boundary);
    guard(si, sl)?;
    if !since_ge(sn, si, sl, bn, bi, bl) {
        return Err(Err::TooEarly);
    }
    Ok(())
}

/// since >= boundary, as fractions. Correct only when both fractions are well formed, which
/// is what the guard below enforces in the fixed build.
fn since_ge(sn: u64, si: u64, sl: u64, bn: u64, bi: u64, bl: u64) -> bool {
    if sn != bn {
        return sn > bn;
    }
    si * bl >= bi * sl
}

/// The pitfall: the fraction is never checked, so 0/0 slips through as zero.
#[cfg(not(feature = "fixed"))]
fn guard(_index: u64, _length: u64) -> Result<(), Err> {
    Ok(())
}

/// The fix: a well formed fraction before the value is read.
#[cfg(feature = "fixed")]
fn guard(index: u64, length: u64) -> Result<(), Err> {
    if length == 0 || index >= length {
        return Err(Err::BadSince);
    }
    Ok(())
}
