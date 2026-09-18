# 09. `since` flags

**Class:** a consensus feature whose encoding is easy to get subtly wrong. **Severity:**
a timing rule that does not hold, or one that can never be satisfied.

## The facts (RFC 0017)

`since` is a `u64` on an input. The top byte carries the flags:

- bit 63: **relative** (1) or absolute (0);
- bits 62 and 61: the metric: `00` block number, `01` epoch, `10` timestamp;
- the remaining 56 bits: the value.

So the top byte is `0x40` for an **absolute timestamp** and `0xC0` for a **relative
timestamp** (`0x80 | 0x40`). Consensus refuses to include the transaction until the
condition holds, which is the whole reason to use it: the script needs no clock, no
header and no self-attestation.

## Where it goes wrong

- Checking only the value and not the flags: a relative delay of sixty seconds and an
  absolute timestamp of sixty seconds since the epoch have the same low bits.
- Checking the flags loosely: a mask that admits the epoch metric where a timestamp was
  meant.
- Forgetting that timestamps in `since` are **seconds** while block headers carry
  **milliseconds**.

## The patterns that hold

Prove a date has passed (a cell that may be cleared after `unlock_at`):

```rust
const ABS_TIMESTAMP_FLAG: u64 = 0x40;
if (since >> 56) != ABS_TIMESTAMP_FLAG { return Err(TooEarly) }
let ts = since & 0x00FF_FFFF_FFFF_FFFF;
if ts < unlock_at { return Err(TooEarly) }
```

Prove a commit cell is old enough (reveal after a minimum delay):

```rust
const REL_TIMESTAMP_FLAG: u64 = 0xC0;
if (since >> 56) != REL_TIMESTAMP_FLAG || (since & 0x00FF_FFFF_FFFF_FFFF) < MIN_DELAY { return Err(TooYoung) }
```

A cell that may change at most once per interval: the cell's age **is** the time since
the last update, because every update recreates it, so a relative `since` on its input
is the whole cooldown rule.

## The test

Both flag bytes wrong (absolute where relative is required, epoch where timestamp is)
must be refused; the boundary value must be tested to the second in both directions. The
harness can set any `since`; consensus only checks it on a real network, so the script
must check the flags itself.

## Runnable

`code/contracts/timed-lock`, test `p09_since_flags`, boundary tested to the second.

## References

- The encoding, normative: [RFC 0017, transaction valid since](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0017-tx-valid-since/0017-tx-valid-since.md)
