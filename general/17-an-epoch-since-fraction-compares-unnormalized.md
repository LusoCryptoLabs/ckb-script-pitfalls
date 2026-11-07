# 17. An epoch `since` fraction compares unnormalized

**Class:** a `since` comparison that reads a malformed fraction as a real one. **Severity:**
a time lock that opens before the moment it names.

## The facts (RFC 0017)

An epoch `since` is an EpochNumberWithFraction: within the low 56 bits, the epoch number
(bits 0 to 23), an index (24 to 39) and a length (40 to 55). The time it names is
`number + index / length`. A well formed fraction has `length >= 1` and `index < length`;
`length` is the number of blocks the epoch is expected to hold, so `0 / 0` is not a point in
time, it is a value with no fraction at all. Consensus admits it and treats it as
`number + 0`, the start of the epoch.

So a script that enforces a boundary of `number + index / length` must compare the input's
`since` against it as a fraction, and must first decide what to do with a fraction that is
not well formed.

## Where it goes wrong

The natural comparison is a cross multiply, which avoids division:

```rust
// since >= boundary, when the numbers are equal
s.index * b.length >= b.index * s.length
```

It is correct only when both lengths are real. Give the `since` a length of zero and both
sides collapse: for a boundary of `1000 + 1/2` and a `since` of `1000 + 0/0`, the numbers are
equal and `0 * 2 >= 1 * 0` is `0 >= 0`, true. The cell opens at the start of epoch 1000, half
an epoch (about two hours) before `1000 + 1/2`, the time it was locked to.

The same shape in the denominator opens it early by other amounts: a boundary of `10 + 1/2`
is met by `10 + 1/3`, because the comparison never brings the fractions to a common base.

The fix is one line before the comparison: a fraction whose `length` is zero (or whose
`index` is not below its `length`) is not a time, and is refused.

```rust
if length == 0 || index >= length { return Err(BadSince) }
```

## Found in deployed scripts

Omnilock compared an epoch `since` this way and corrected it in
[280dd97](https://github.com/cryptape/omnilock/commit/280dd97ac0930ea1ed766a18d49478229347c832);
the system `secp256k1_blake160_multisig_all` has the same comparison, with the fix open but
not yet deployed in
[ckb-system-scripts#106](https://github.com/nervosnetwork/ckb-system-scripts/pull/106). The
related liveness side of an epoch `since` (a payment refused because two fractions will not
compare) is [omnilock#12](https://github.com/nervosnetwork/omnilock/issues/12).

## The test

The length-zero `since` must be accepted by the script as first written and refused by the
corrected one; a genuinely early fraction (`1000 + 0/1` against `1000 + 1/2`) must be refused
by both, so the test shows the comparison is otherwise sound and only the unnormalized
fraction is the hole; the on-time boundary and a later epoch pass on both.

## Runnable

`code/contracts/epoch-lock`, test `p17_epoch_since_fraction_compares_unnormalized`.

## References

- The encoding, normative: [RFC 0017, transaction valid since](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0017-tx-valid-since/0017-tx-valid-since.md)
