# 02. A typed output meets the capacity

**Class:** payment verified by capacity alone. **Severity:** burn or ransom lever on the
payee's money; theft where the payee is a counterparty.

## The pattern

The payment check reads each output's lock hash and capacity:

```rust
if lock_hash == payee && capacity >= owed { paid = true }
```

Nothing looks at the output's **type script**. So the payer attaches one: an always-fail
type, a joint-custody type, a Nervos DAO lockup. The capacity is there, the lock is the
payee's, and the payee cannot spend the cell alone, or at all. The check is satisfied and
the money is not theirs.

Found twice in the same codebase, five days apart: first on a lock's payout to its
payee (High: the payee gave up the asset for coins they could not move), then on a
protocol fee to its treasury (Medium: a burn or a ransom on what the protocol earns,
since the payer still parts with the CKB). Nobody had asked whether the fix to the first
applied to the second.

## The fix

Only a **pure** output counts as payment: no type script.

```rust
fn is_pure_output(i: usize) -> Result<bool, Error> {
    Ok(load_cell_type_hash(i, Source::Output)?.is_none())
}
```

Apply it to every sum that decides whether money arrived: the fee, a third party's cut,
the payee's leg, the treasury's leg. Search the codebase for the shape rather than waiting
for the third instance.

## The test

The honest transaction with the payee's output carrying a foreign type script must be
refused with the "unpaid" error, not a structural one. The control is the same
transaction with a pure output.

The harness may not be able to express "a plain output that nevertheless carries a
type": the one that found this had to grow a `typed` flag first. A harness that cannot
build the attack is a harness that has never tested it.

## Runnable

`code/contracts/fee-type`, test `p02_a_typed_output_meets_the_capacity`.

## References

- Why a type script on an output binds its future spending: [RFC 0022](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md)
