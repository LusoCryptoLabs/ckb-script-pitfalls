//! `payload-type`: the cell holds a hash, the witness holds the bytes.
//!
//! Keeping a large payload in the witness and only its blake2b in the cell keeps the
//! cell small (capacity is rent). The script must then find the payload, and the
//! convention is: for an output of this type at absolute index `i`, the bytes live in
//! `witnesses[i].output_type`.
//!
//! **10, witnesses by output index.** `load_witness_args(i, Source::Input)` is a plain
//! `witnesses.get(i)`: there is no "witness of output i", only the list, indexed the
//! same way from either side. So the script walks OUTPUTS by absolute index, reads the
//! witness at that index, and compares. A payload in the wrong slot, or a client that
//! sized the witness list by the input count and left slot `i` empty, is a mismatch and
//! the transaction fails: the convention fails closed as long as the check is per
//! absolute output index and not per group index.
#![no_std]
#![no_main]

use blake2b_ref::Blake2bBuilder;
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_data, load_cell_type_hash, load_script_hash, load_witness_args},
};

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

#[repr(i8)]
enum Err {
    Encoding = 1,
    BadData = 2,
    /// The witness at this output's index does not hash to the cell's data.
    Mismatch = 3,
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(e) => e as i8,
    }
}

fn run() -> Result<(), Err> {
    let me = load_script_hash().map_err(|_| Err::Encoding)?;
    let mut i = 0usize;
    loop {
        match load_cell_type_hash(i, Source::Output) {
            Ok(Some(h)) if h == me => {
                let data = load_cell_data(i, Source::Output).map_err(|_| Err::Encoding)?;
                if data.len() != 32 {
                    return Err(Err::BadData);
                }
                // The witness at the OUTPUT's absolute index, read through Source::Input,
                // which is where the list lives.
                let wa = load_witness_args(i, Source::Input).map_err(|_| Err::Mismatch)?;
                let payload = wa.output_type().to_opt().ok_or(Err::Mismatch)?.raw_data();
                if blake2b_256(payload.as_ref())[..] != data[..] {
                    return Err(Err::Mismatch);
                }
                i += 1;
            }
            Ok(_) => i += 1,
            Err(SysError::IndexOutOfBound) => return Ok(()),
            Err(_) => return Err(Err::Encoding),
        }
    }
}

fn blake2b_256(b: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut h = Blake2bBuilder::new(32).personal(b"ckb-default-hash").build();
    h.update(b);
    h.finalize(&mut out);
    out
}
