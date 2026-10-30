# 16. An allowance anyone can claim

**Class:** a discount summed from cells anyone can create. **Severity:** High: the owner's
other cells are taken whole.

## The pattern

A lock opens with no signature as long as its owner is paid back what its cells hold, less
an allowance each cell states: the fee a relayer may keep for the work. Done right by entry
01, the cells of one owner are counted together across the transaction:

```rust
for each input under this lock code whose args name the same owner {
    held += capacity;
    allowed += args.allowance;
}
if paid_to(owner) + allowed < held { return Err(Underpaid) }
```

Nothing runs when a cell is created under a lock, so anyone can make one that names anyone
as its owner. A stranger makes a cell for the victim whose allowance is larger than
everything in the transaction, and spends it beside the victim's real cell. `allowed` now
covers both, `paid_to(owner)` may be zero, and the victim's CKB goes where the stranger
likes. The stranger's own cell goes back to them too.

The per-owner sum that entry 01 asks for is what makes the forged cell count: a check per
script group would have kept the victim's group to its own allowance, and fallen to 01
instead.

Found in the claim tickets of a token reward pool on a public test network: a ticket anyone
may fund for anyone, opened without a signature by whoever claims, its fee allowance summed
over the owner's tickets. Reproduced against the deployed binary: a forged ticket with the
largest allowance, beside a real one of 1,000 CKB, let the claim pay the owner 300 CKB and
the runner the rest.

## The fix

A cell's allowance counts up to what that cell holds, never more:

```rust
allowed += min(args.allowance, capacity);
```

A cell anyone made then adds at most its own CKB to what may be kept, so it can only cost
its maker. The same goes for any term a cell states for its owner's cost or benefit (a
discount, a rebate, a deposit credit): bounded by what that cell brings.

## The question to ask

**Which numbers in a sum come from cells anyone can create, and can one be large enough to
cancel the others?** Anything summed per owner, per account or per pool across inputs,
where each input's terms were written by whoever made it.

## The test

Alice's cell of 1,000 CKB allowing 1 CKB, and a cell Mallory made for Alice of 100 CKB
allowing everything, spent together with all 1,100 CKB to Mallory: accepted as first
written, refused with the fix (2). Controls on both builds: Alice paid back 999 CKB with
Mallory's cell going back to Mallory; Alice's cell alone, paid back; and the same a shannon
short, refused.

## Runnable

`code/contracts/allowance-lock`, test `p16_an_allowance_anyone_can_claim`.

## Related

- [01](01-a-payment-sum-is-safe-only-against-itself.md): the per-owner sum this builds on.
- [02](02-a-typed-output-meets-the-capacity.md): what counts as paid back; the toy counts
  plain outputs only, so that 02 is not what the test shows.
