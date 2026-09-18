# 01. A payment sum is safe only against itself

**Class:** cross-instance and cross-script double counting. **Severity when it applies:**
the smaller fee is never paid; with more instances, whole fees vanish.

## The pattern

A script needs to be paid. The natural check is:

```rust
fn paid_to(lock_hash: &[u8; 32]) -> u64 {
    // sum the capacity of every output under lock_hash
}
if paid_to(&TREASURY) < fee { return Err(Unpaid) }
```

It is correct for one instance of one script in one transaction. It is wrong the moment
anything else in the transaction asks the same question of the same outputs, because
**every instance reads the whole transaction and each is satisfied by the same total.**

Three shapes, each found separately, each the same mistake:

1. **Two instances of the same lock, different args.** A lock with the amount it is owed
   in its args: two cells of one payee at two amounts are two script hashes. Each counted
   `amount * count(my exact hash)`, both read the same outputs, and one payment for the
   dearer cell satisfied the cheaper one, which was consumed for free.
2. **Two instances, two payees, one shared leg.** The payee's leg is per payee (their
   payout is under their own lock, so it cannot be shared). The fee's leg is not:
   with two payees in one transaction, one fee-sized output satisfied both instances,
   each seeing enough for itself and nobody seeing the sum.
3. **Two different scripts.** An action of script A owing a large fee to a treasury,
   batched with an action of script B owing a small fee to the same treasury: one output
   of the large fee satisfied both. Neither knows the other is running. Reproduced in
   the VM: accepted, the small fee never paid. The control, paying both, also passes, so
   the shape is constructible and the failure is specifically the double count.

The same shape reappears one level up whenever two **deployments** of one type script
(different args, so two script groups) share a fee lock: two actions, one fee.

## Why the usual mitigation is wrong

"Not exploitable today because only one deployment exists" was written and was already
false, because a *second script* counting the same outputs does not have to be a second
deployment. It was checked against the wrong noun.

## The fix

Some script has to demand the **total** obligation in the transaction rather than its own
share, which means reading the other instances' (or the other script's) cells and
computing what they owe. One-sided is enough, since the strictest requirement binds:

- within one script: sum what is owed over **all** of its instances in the transaction
  (each priced by its own args), so every instance demands one indivisible figure;
- across two scripts: the one that can do the arithmetic reads the other's inputs (by
  code hash, from its args) and adds what they owe to its own requirement. This couples
  the two contracts, so it is a decision, not a patch. A baked code hash fails **open** if
  it goes stale; pin it with a test against the deployment record.
- across deployments: a fee lock per deployment. It costs a key.

Mirror any exemption exactly. If the lock demands nothing for a cell whose payee is
present (they are taking it back, not being paid), the other script must apply the same
predicate or it refuses honest batches. Duplicated logic, and that is the real cost.

## The test

Build the two-instance (or two-script) transaction with **one** payment that satisfies
the larger requirement only. It must be refused. The control pays both and must pass.
Note that every test file in the project deployed exactly one contract per `Context`
until this was found: the shape that costs the most had no home in the suite.

## The rule to remember

A check of the form "sum what is paid to L" is safe only against other instances of the
**same script**, because that is all it can see. Two different scripts needing payment
to the same lock in one transaction each read the whole sum and each are satisfied by it.

## Runnable

`code/contracts/fee-type`, tests `p01_one_payment_answers_two_instances` and
`p13_the_fee_rule_as_a_property` in `code/tests/tests/pitfalls.rs`.

## References

- Script groups and how a script sees a transaction: [RFC 0022, transaction structure](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md)
- Syscalls a script reads the transaction with: [RFC 0009, VM syscalls](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0009-vm-syscalls/0009-vm-syscalls.md)
