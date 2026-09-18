# 13. Examples are not properties

**Class:** how a suite can be large and green and guard nothing. **Severity:** it is how
a real hole got through 171 tests and eight review passes.

## The pattern

Every test describes a transaction somebody already imagined, and asks *does this
behave?* None asks *is there any transaction in this space that behaves wrongly?*, which
is the only question an attacker asks. A list of examples cannot answer it, however long
the list.

The fix for that hole made eight existing tests fail. They were right to: every one of them
encoded the old split. A suite that agrees with the code in detail is a suite that will
agree with the code's mistakes in detail.

## What a property looks like here

For a lock paid in two legs, over a grid of 225 combinations (nine amounts against five
payout offsets on each leg), assert an **if and only if**: the lock opens exactly when the
payee receives at least `amount - fee + deposit` and the treasury at least `fee`.

With the fix reverted and the contract rebuilt, the property fails on 66 of the 225 and
the second assertion says it in words: *"at amount 61 the lock accepted a payment of 64
CKB to the payee, less than the 100 CKB deposit they had parked"*.

For the type script beside it: eight properties, and every guard they cover was disabled
in turn to watch the right property fail and only that one.

| guard broken | what failed | what stayed green |
|---|---|---|
| capacity check disabled | both capacity properties, naming four actions | everything else |
| fee check weakened to half | the fee property, at the one-shannon case | everything else |
| a time threshold ignored | the time property, and only it | six others |
| range arithmetic admitting an endpoint | the range property, naming the function and the case, plus five downstream | the time property and the endpoints |

A property that catches its own guard and ignores the others means something specific
when it passes. One that catches everything means the suite ran.

## Three things this taught

- **A control that is not itself checked is not a control.** The first attempt reported
  that the property did *not* catch the hole, because the rebuild had failed and the test ran
  against the already-fixed binary.
- **An independent definition earns its keep by being wrong in public.** Two of the
  properties were wrong before the contract was: one asserted a field that does not exist
  (the value is derived, not stored, which is a stronger guarantee than a check); the
  other wrote out a range rule independently and got the degenerate one-element case
  wrong, reporting 65,536 disagreements with a contract that was right.
- **A fuzz that lives in the repository beats a corpus on somebody's machine.** 105,000
  seeded inputs through every parser, asserting no panic, exact round-trip, and that
  every short prefix is refused rather than interpreted. Weaker than cargo-fuzz at deep
  paths, stronger at being run.

## Runnable

`p13_the_fee_rule_as_a_property` in `code/tests/tests/pitfalls.rs`: the fee rule as an
if-and-only-if over a grid, asserted on the fixed build, and the same grid run against the
vulnerable build to name every point where it is wrong.
