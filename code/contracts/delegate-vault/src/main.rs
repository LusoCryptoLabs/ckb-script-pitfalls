//! `delegate-vault`: a vault anyone may top up, under a lock that defers to its type.
//!
//! One binary, two roles, told apart by `args[0]`; the other thirty-two bytes are an id.
//!   * **role 0, the type**: a vault may be created freely and afterwards only grow: one
//!     in, one out, and the output holds at least the input's capacity. Anyone may do
//!     this; a vault you can pay into is the point.
//!   * **role 1, the lock**: opens whenever an input carries the role 0 type of the same
//!     id. It judges nothing and leaves that to the type, a common and sound design for
//!     the vault's own cells.
//!
//! **15, a lock anyone can open is an owner key.** The role 1 lock is in every vault
//! transaction, and anyone can build one. xUDT owner mode asks only for an input under
//! the owner lock, so a token whose owner is the hash of the role 1 lock is minted by
//! whoever tops up the vault. Fixed: a vault transaction may not raise any xUDT's supply.
#![no_std]
#![no_main]

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{packed::Script, prelude::*},
    error::SysError,
    high_level::{load_cell_capacity, load_cell_type, load_script},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

/// Data hash of the mainnet xUDT code cell, referenced with hash type data1.
#[cfg(feature = "fixed")]
const XUDT_DATA_HASH: [u8; 32] = [
    0x50, 0xbd, 0x8d, 0x66, 0x80, 0xb8, 0xb9, 0xcf, 0x98, 0xb7, 0x3f, 0x3c, 0x08, 0xfa, 0xf8, 0xb2,
    0xa2, 0x19, 0x14, 0x31, 0x19, 0x54, 0x11, 0x8a, 0xd6, 0x60, 0x9b, 0xe6, 0xe7, 0x8a, 0x1b, 0x95,
];

#[repr(i8)]
#[allow(dead_code)] // ForeignMint exists only in the fixed build
enum Err {
    Encoding = 1,
    Arity = 2,
    /// The vault would hold less than it did.
    Shrunk = 3,
    /// The lock was asked to open with no vault among the inputs.
    NoVault = 4,
    /// A vault transaction raised the supply of an xUDT.
    ForeignMint = 5,
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
    let script = load_script().map_err(|_| Err::Encoding)?;
    let args = script.args().raw_data();
    if args.len() != 33 {
        return Err(Err::Encoding);
    }
    match args[0] {
        0 => vault_type(),
        1 => vault_lock(&script, &args[1..]),
        _ => Err(Err::Encoding),
    }
}

fn vault_type() -> Result<(), Err> {
    no_foreign_mint()?;
    let ins = count(Source::GroupInput)?;
    let outs = count(Source::GroupOutput)?;
    match (ins, outs) {
        (0, _) => Ok(()), // create
        (1, 1) => {
            let cin = load_cell_capacity(0, Source::GroupInput).map_err(|_| Err::Encoding)?;
            let cout = load_cell_capacity(0, Source::GroupOutput).map_err(|_| Err::Encoding)?;
            if cout < cin {
                return Err(Err::Shrunk);
            }
            Ok(())
        }
        _ => Err(Err::Arity),
    }
}

/// Opens when some input carries the role 0 type of the same id. Judgement is the type's.
fn vault_lock(script: &Script, id: &[u8]) -> Result<(), Err> {
    let mut i = 0usize;
    loop {
        match load_cell_type(i, Source::Input) {
            Ok(Some(t)) => {
                let a = t.args().raw_data();
                if t.code_hash().as_slice() == script.code_hash().as_slice()
                    && t.hash_type().as_slice() == script.hash_type().as_slice()
                    && a.len() == 33
                    && a[0] == 0
                    && a[1..] == *id
                {
                    return Ok(());
                }
                i += 1;
            }
            Ok(None) => i += 1,
            Err(SysError::IndexOutOfBound) => return Err(Err::NoVault),
            Err(_) => return Err(Err::Encoding),
        }
    }
}

/// The pitfall: the vault judges its own cells and nothing else in the transaction.
#[cfg(not(feature = "fixed"))]
fn no_foreign_mint() -> Result<(), Err> {
    Ok(())
}

/// The fix: no xUDT leaves a vault transaction with more supply than it came in with.
#[cfg(feature = "fixed")]
fn no_foreign_mint() -> Result<(), Err> {
    use ckb_std::high_level::load_cell_type_hash;
    for i in 0..count(Source::Output)? {
        let Some(t) = load_cell_type(i, Source::Output).map_err(|_| Err::Encoding)? else {
            continue;
        };
        if t.code_hash().as_slice() != XUDT_DATA_HASH || t.hash_type().as_slice() != [2] {
            continue;
        }
        let token = load_cell_type_hash(i, Source::Output).map_err(|_| Err::Encoding)?;
        // Judge each token once, at its first output.
        let mut first = true;
        for j in 0..i {
            if load_cell_type_hash(j, Source::Output).map_err(|_| Err::Encoding)? == token {
                first = false;
                break;
            }
        }
        if first && supply(token, Source::Output)? > supply(token, Source::Input)? {
            return Err(Err::ForeignMint);
        }
    }
    Ok(())
}

#[cfg(feature = "fixed")]
fn supply(token: Option<[u8; 32]>, source: Source) -> Result<u128, Err> {
    use ckb_std::high_level::{load_cell_data, load_cell_type_hash};
    let mut sum = 0u128;
    for i in 0..count(source)? {
        if load_cell_type_hash(i, source).map_err(|_| Err::Encoding)? != token {
            continue;
        }
        let d = load_cell_data(i, source).map_err(|_| Err::Encoding)?;
        let amount = d.get(..16).ok_or(Err::Encoding)?;
        let amount = u128::from_le_bytes(amount.try_into().map_err(|_| Err::Encoding)?);
        sum = sum.checked_add(amount).ok_or(Err::Encoding)?;
    }
    Ok(sum)
}
