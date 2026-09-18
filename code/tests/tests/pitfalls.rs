//! One test per pitfall, each in the same shape: the exploiting transaction is ACCEPTED by
//! the script as first written and REFUSED by the corrected one, with the error code the
//! entry names; and a control, the honest transaction of the same shape, passes on both.
//!
//! Run `make test` from `code/`. Every lock here is ckb-testtool's always-success with
//! distinct args, so a "wallet" is an args string and signing is not the subject.
use blake2b_ref::Blake2bBuilder;
use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::TransactionBuilder,
    packed::{BytesOpt, CellInput, CellOutput, Script, WitnessArgs},
    prelude::*,
};
use ckb_testtool::context::Context;
use tests::{load, Build};
use Build::{Fixed, Vulnerable};

const MAX_CYCLES: u64 = 50_000_000;
const CKB: u64 = 100_000_000;
/// What every input cell holds: plenty, so capacity is never the reason a test fails
/// unless the test is about capacity.
const CAP: u64 = 1_000 * CKB;

fn verify(ctx: &mut Context, tx: ckb_testtool::ckb_types::core::TransactionView) -> Result<u64, String> {
    let tx = ctx.complete_tx(tx);
    ctx.verify_tx(&tx, MAX_CYCLES).map_err(|e| format!("{e:?}"))
}

/// Assert the script failed FOR THE REASON under test, not merely that it failed.
fn refused(res: &Result<u64, String>, code: u8, what: &str) {
    let e = res.as_ref().err().unwrap_or_else(|| panic!("{what}: expected a failure, it passed"));
    assert!(e.contains(&format!("error code {code}")), "{what}: wanted error {code}, got {e}");
}

fn passed(res: &Result<u64, String>, what: &str) {
    assert!(res.is_ok(), "{what}: expected to pass, got {res:?}");
}

fn bytes_opt(b: &[u8]) -> BytesOpt {
    BytesOpt::new_builder().set(Some(Bytes::from(b.to_vec()).pack())).build()
}

fn blake2b_256(b: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut h = Blake2bBuilder::new(32).personal(b"ckb-default-hash").build();
    h.update(b);
    h.finalize(&mut out);
    out
}

/// A lock that anyone can satisfy, told apart by its args. `wallet("alice")` is one
/// person, `wallet("treasury")` another; what matters is that their hashes differ.
fn wallet(ctx: &mut Context, who: &str) -> Script {
    let lock_op = ctx.deploy_cell(ALWAYS_SUCCESS.clone());
    ctx.build_script(&lock_op, Bytes::from(who.as_bytes().to_vec())).expect("lock")
}

// --- 01 and 02: fee-type ----------------------------------------------------------

/// An output paying the treasury: its capacity, and whether a foreign type script is
/// attached to it (the shape of pitfall 02).
#[derive(Clone, Copy)]
struct Pay {
    ckb: u64,
    typed: bool,
}

/// Spend one fee-type cell per entry of `fees` (each at its own fee, so each is its own
/// script group), paying the treasury as `pays` says. Returns the verifier's verdict.
fn fee_tx(build: Build, fees: &[u64], pays: &[Pay]) -> Result<u64, String> {
    let mut ctx = Context::default();
    let fee_op = ctx.deploy_cell(load(build, "fee-type"));
    let treasury = wallet(&mut ctx, "treasury");
    let treasury_hash = treasury.calc_script_hash();
    let holder = wallet(&mut ctx, "holder");
    let hostile = wallet(&mut ctx, "hostile-type");

    let mut inputs = Vec::new();
    for &fee in fees {
        let mut args = treasury_hash.as_bytes().to_vec();
        args.extend_from_slice(&(fee * CKB).to_le_bytes());
        let t = ctx.build_script(&fee_op, Bytes::from(args)).expect("fee type");
        let op = ctx.create_cell(
            CellOutput::new_builder().capacity(CAP.pack()).lock(holder.clone()).type_(Some(t).pack()).build(),
            Bytes::new(),
        );
        inputs.push(CellInput::new_builder().previous_output(op).build());
    }
    let mut outputs = Vec::new();
    for p in pays {
        let b = CellOutput::new_builder().capacity((p.ckb * CKB).pack()).lock(treasury.clone());
        outputs.push(if p.typed { b.type_(Some(hostile.clone()).pack()).build() } else { b.build() });
    }
    // The holder's change, so a transaction paying nobody still has an output.
    outputs.push(CellOutput::new_builder().capacity((100 * CKB).pack()).lock(holder.clone()).build());
    let n = outputs.len();
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(vec![Bytes::new().pack(); n])
        .build();
    verify(&mut ctx, tx)
}

#[test]
fn p01_one_payment_answers_two_instances() {
    // Two cells at fees 100 and 50: two script groups reading the same outputs. One
    // output of 100 covers the larger fee. As first written, both instances are
    // satisfied and 50 CKB are never paid.
    let attack = |b| fee_tx(b, &[100, 50], &[Pay { ckb: 100, typed: false }]);
    passed(&attack(Vulnerable), "01 vulnerable: one output of 100 answers fees of 100 and 50");
    refused(&attack(Fixed), 3, "01 fixed: the total of 150 is demanded");
    // The control: paying both is accepted on either build.
    let honest = |b| fee_tx(b, &[100, 50], &[Pay { ckb: 150, typed: false }]);
    passed(&honest(Vulnerable), "01 control on vulnerable");
    passed(&honest(Fixed), "01 control on fixed");
}

#[test]
fn p02_a_typed_output_meets_the_capacity() {
    // The fee output carries a foreign type script: right lock, right capacity, and the
    // treasury cannot spend it alone.
    let attack = |b| fee_tx(b, &[100], &[Pay { ckb: 100, typed: true }]);
    passed(&attack(Vulnerable), "02 vulnerable: a typed output counts as payment");
    refused(&attack(Fixed), 3, "02 fixed: only a pure output counts");
    let honest = |b| fee_tx(b, &[100], &[Pay { ckb: 100, typed: false }]);
    passed(&honest(Vulnerable), "02 control on vulnerable");
    passed(&honest(Fixed), "02 control on fixed");
}

// --- 03 and 07: owned-type --------------------------------------------------------

fn owner_prefix(s: &Script) -> [u8; 20] {
    let mut p = [0u8; 20];
    p.copy_from_slice(&s.calc_script_hash().as_bytes()[..20]);
    p
}

fn owned_data(owner: [u8; 20], value: u64) -> Bytes {
    let mut d = owner.to_vec();
    d.extend_from_slice(&value.to_le_bytes());
    Bytes::from(d)
}

/// Create one owned-type cell. `owner_is_cell_lock` sets the owner to the prefix of the
/// cell's own always-success lock, which is the value pitfall 03's guard exists to refuse.
fn create_tx(build: Build, owner_is_cell_lock: bool) -> Result<u64, String> {
    let mut ctx = Context::default();
    let type_op = ctx.deploy_cell(load(build, "owned-type"));
    let t = ctx.build_script(&type_op, Bytes::new()).expect("type");
    let cell_lock = wallet(&mut ctx, "the-cell-lock");
    let alice = wallet(&mut ctx, "alice");
    let owner = if owner_is_cell_lock { owner_prefix(&cell_lock) } else { owner_prefix(&alice) };
    // Alice pays for the cell from a plain cell of hers.
    let funding = ctx.create_cell(CellOutput::new_builder().capacity(CAP.pack()).lock(alice.clone()).build(), Bytes::new());
    let tx = TransactionBuilder::default()
        .input(CellInput::new_builder().previous_output(funding).build())
        .output(CellOutput::new_builder().capacity((200 * CKB).pack()).lock(cell_lock).type_(Some(t).pack()).build())
        .output_data(owned_data(owner, 1).pack())
        .build();
    verify(&mut ctx, tx)
}

/// One owned-type cell (owner alice, value 1, capacity `cap_in`) acted on by `signer`
/// with `action`, leaving a cell with `value_out` and `cap_out`.
fn act_tx(build: Build, action: &[u8], signer: &str, owner_is_cell_lock: bool, cap_in: u64, cap_out: u64, value_out: u64) -> Result<u64, String> {
    let mut ctx = Context::default();
    let type_op = ctx.deploy_cell(load(build, "owned-type"));
    let t = ctx.build_script(&type_op, Bytes::new()).expect("type");
    let cell_lock = wallet(&mut ctx, "the-cell-lock");
    let alice = wallet(&mut ctx, "alice");
    let who = wallet(&mut ctx, signer);
    let owner = if owner_is_cell_lock { owner_prefix(&cell_lock) } else { owner_prefix(&alice) };
    let cell = ctx.create_cell(
        CellOutput::new_builder().capacity(cap_in.pack()).lock(cell_lock.clone()).type_(Some(t.clone()).pack()).build(),
        owned_data(owner, 1),
    );
    let signers_cell = ctx.create_cell(CellOutput::new_builder().capacity(CAP.pack()).lock(who.clone()).build(), Bytes::new());
    let witness = WitnessArgs::new_builder().input_type(bytes_opt(action)).build().as_bytes().pack();
    let tx = TransactionBuilder::default()
        .input(CellInput::new_builder().previous_output(cell).build())
        .input(CellInput::new_builder().previous_output(signers_cell).build())
        .output(CellOutput::new_builder().capacity(cap_out.pack()).lock(cell_lock).type_(Some(t).pack()).build())
        .output_data(owned_data(owner, value_out).pack())
        // The signer's change; the difference between cap_in and cap_out lands here.
        .output(CellOutput::new_builder().capacity((CAP + cap_in - cap_out).pack()).lock(who).build())
        .output_data(Bytes::new().pack())
        .witness(witness)
        .build();
    verify(&mut ctx, tx)
}

#[test]
fn p03_a_truncated_compare_fails_open() {
    // The owner is the cell's own always-success lock, which the guard is there to refuse.
    passed(&create_tx(Vulnerable, true), "03 vulnerable: 20 bytes never equal 32, the guard never fires");
    refused(&create_tx(Fixed, true), 4, "03 fixed: the prefix compare refuses it");
    passed(&create_tx(Vulnerable, false), "03 control on vulnerable");
    passed(&create_tx(Fixed, false), "03 control on fixed");
    // And what such a cell is: public property. A stranger's edit is authorized by the
    // cell's own input, because that input carries the always-success lock the owner
    // field names. The fixed build never lets the cell exist, so only the vulnerable
    // build has this consequence.
    passed(
        &act_tx(Vulnerable, b"edit", "mallory", true, 200 * CKB, 200 * CKB, 1),
        "03 consequence: anyone edits a cell owned by the always-success lock",
    );
    // For an honestly owned cell, a stranger's edit is refused on both.
    refused(&act_tx(Vulnerable, b"edit", "mallory", false, 200 * CKB, 200 * CKB, 1), 8, "03 stranger edit on vulnerable");
    refused(&act_tx(Fixed, b"edit", "mallory", false, 200 * CKB, 200 * CKB, 1), 8, "03 stranger edit on fixed");
}

#[test]
fn p07_capacity_is_unguarded_on_permissionless_actions() {
    // A stranger bumps the value (allowed) and leaves the cell with less capacity than it
    // had (the owner's rent), keeping the difference as change.
    let attack = |b| act_tx(b, b"bump", "mallory", false, 200 * CKB, 100 * CKB, 2);
    passed(&attack(Vulnerable), "07 vulnerable: every byte of data pinned, the money free");
    refused(&attack(Fixed), 5, "07 fixed: the surviving cell may not shrink");
    let honest = |b| act_tx(b, b"bump", "mallory", false, 200 * CKB, 200 * CKB, 2);
    passed(&honest(Vulnerable), "07 control on vulnerable");
    passed(&honest(Fixed), "07 control on fixed");
}

// --- 09: timed-lock ---------------------------------------------------------------

fn timed_tx(build: Build, unlock_at: u64, since: u64) -> Result<u64, String> {
    let mut ctx = Context::default();
    let lock_op = ctx.deploy_cell(load(build, "timed-lock"));
    let lock = ctx.build_script(&lock_op, Bytes::from(unlock_at.to_le_bytes().to_vec())).expect("lock");
    let bob = wallet(&mut ctx, "bob");
    let cell = ctx.create_cell(CellOutput::new_builder().capacity(CAP.pack()).lock(lock).build(), Bytes::new());
    let tx = TransactionBuilder::default()
        .input(CellInput::new_builder().since(since.pack()).previous_output(cell).build())
        .output(CellOutput::new_builder().capacity(CAP.pack()).lock(bob).build())
        .output_data(Bytes::new().pack())
        .build();
    verify(&mut ctx, tx)
}

#[test]
fn p09_since_flags() {
    let unlock_at: u64 = 1_800_000_000;
    let absolute = (0x40u64 << 56) | unlock_at;
    let relative = (0xC0u64 << 56) | unlock_at;
    // A relative timestamp carrying the same number: the low bits pass the comparison,
    // and consensus reads it as "this input is unlock_at seconds old", not a date.
    passed(&timed_tx(Vulnerable, unlock_at, relative), "09 vulnerable: the flags are never read");
    refused(&timed_tx(Fixed, unlock_at, relative), 3, "09 fixed: not an absolute timestamp");
    passed(&timed_tx(Vulnerable, unlock_at, absolute), "09 control on vulnerable");
    passed(&timed_tx(Fixed, unlock_at, absolute), "09 control on fixed");
    // And the boundary, to the second, on the fixed build.
    refused(&timed_tx(Fixed, unlock_at, absolute - 1), 2, "09 one second early");
    passed(&timed_tx(Fixed, unlock_at, absolute + 1), "09 one second late");
}

// --- 10: payload-type -------------------------------------------------------------

/// Two payload cells created from one plain input, with the witness list in the given
/// order. The honest order puts each payload at its cell's output index.
fn payload_tx(payloads: [&[u8]; 2], witness_order: [usize; 2]) -> Result<u64, String> {
    let mut ctx = Context::default();
    let type_op = ctx.deploy_cell(load(Fixed, "payload-type"));
    let t = ctx.build_script(&type_op, Bytes::new()).expect("type");
    let alice = wallet(&mut ctx, "alice");
    let funding = ctx.create_cell(CellOutput::new_builder().capacity(CAP.pack()).lock(alice.clone()).build(), Bytes::new());
    let mut tb = TransactionBuilder::default().input(CellInput::new_builder().previous_output(funding).build());
    for p in payloads {
        tb = tb
            .output(CellOutput::new_builder().capacity((200 * CKB).pack()).lock(alice.clone()).type_(Some(t.clone()).pack()).build())
            .output_data(Bytes::from(blake2b_256(p).to_vec()).pack());
    }
    for &k in &witness_order {
        tb = tb.witness(WitnessArgs::new_builder().output_type(bytes_opt(payloads[k])).build().as_bytes().pack());
    }
    verify(&mut ctx, tb.build())
}

#[test]
fn p10_witnesses_by_output_index() {
    // More outputs than inputs: the second output's witness is at index 1, past the one
    // input. The list is padded to the output count and it passes.
    passed(&payload_tx([b"first", b"second"], [0, 1]), "10 payloads at their output indices");
    // The same two payloads swapped: each cell's data no longer matches the slot at its
    // own index, so the convention fails closed.
    refused(&payload_tx([b"first", b"second"], [1, 0]), 3, "10 payloads in the wrong slots");
}

// --- 13: a property instead of examples -------------------------------------------

#[test]
fn p13_the_fee_rule_as_a_property() {
    // Over a grid rather than a list: the fixed script accepts exactly when the treasury
    // is paid at least the fee in a PURE output. The same grid run against the
    // vulnerable build names every point where it disagrees, which is how a property
    // proves it can catch what it claims to.
    let fees = [1u64, 63, 100, 1_000];
    let mut wrong_on_vulnerable = Vec::new();
    for &fee in &fees {
        for paid in [fee - 1, fee, fee + 1] {
            for typed in [false, true] {
                let should_pass = !typed && paid >= fee;
                let fixed = fee_tx(Fixed, &[fee], &[Pay { ckb: paid, typed }]).is_ok();
                assert_eq!(fixed, should_pass, "fixed build at fee {fee}, paid {paid}, typed {typed}");
                let vulnerable = fee_tx(Vulnerable, &[fee], &[Pay { ckb: paid, typed }]).is_ok();
                if vulnerable != should_pass {
                    wrong_on_vulnerable.push((fee, paid, typed));
                }
            }
        }
    }
    // Every typed-and-sufficient point is wrongly accepted by the vulnerable build, and
    // nothing else is.
    assert_eq!(wrong_on_vulnerable.len(), fees.len() * 2, "the vulnerable build is wrong at {wrong_on_vulnerable:?}");
    assert!(wrong_on_vulnerable.iter().all(|&(fee, paid, typed)| typed && paid >= fee));
}
