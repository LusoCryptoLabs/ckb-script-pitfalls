# 04. A new cell's lock is not pinned

**Class:** availability, plus two fee dodges from the same gap. **Severity:** Critical
where the type script maintains a structure across cells.

## The pattern

A type script that guards a structure spanning many cells (here a sorted ring of names,
where inserting spends a predecessor cell) is careful about **data**: every field of the
predecessor is compared byte for byte, the new cell's fields are validated. Its **lock** is
read once, to feed a different check, and never pinned.

So anyone registers a cell under an unspendable lock. The type script cannot object: it
knows nothing about locks. That cell now sits in the ring, and the ring's invariant says
the next insertion in its range must spend it as a predecessor. It can never be spent.
Its whole range is frozen, forever, for the cost of one registration.

The same gap, used differently: set the new cell's lock to the **treasury's**, and the
cell's own refundable rent counts as the registration fee (the fee check sums outputs by
lock). Set it to an inviter's, and the rent counts as the referral cut. Three findings,
one missing line.

## The fix

Every output cell of the type must wear the same lock as the input cells of the type,
which by induction from genesis is the one lock the design intended:

```rust
let account_lock = lock_hash(&inputs[0], Source::Input)?;
for c in &outputs {
    if lock_hash(c, Source::Output)? != account_lock { return Err(CellLockNotAccountLock) }
}
```

Genesis is the one exemption: it has no input of the type, and it runs once, gated by a
token that cannot be recreated.

## The test

Register with the new cell under a foreign lock: refused with the lock error, not a fee
error. Control: the same transaction under the right lock passes the pin and fails on
whatever comes next in the validator (which proves the transaction is well formed and the
refusal was the pin). Both were also proven on the live network before the fix was called
done.

## The question to ask

For every cell the script creates: **who decided its lock, and what stops them choosing
one nobody can spend?**

## Where it lives

The contracts are public at [cells-contracts](https://github.com/LusoCryptoLabs/cells-contracts). The tests that pin it, in [`tests/tests/account_cell.rs`](https://github.com/LusoCryptoLabs/cells-contracts/blob/main/tests/tests/account_cell.rs): `register_rejects_a_new_cell_under_a_foreign_lock` is the refusal, and `register_rejects_repointing_the_predecessor_lock` and `transfer_rejects_a_foreign_cell_lock` are the same pin on the other two paths a lock could move.
