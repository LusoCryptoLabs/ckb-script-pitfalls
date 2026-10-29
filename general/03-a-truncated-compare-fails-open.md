# 03. A truncated compare fails open

**Class:** a guard that can never fire. **Severity:** whatever the guard protected; here,
an asset anybody could edit and transfer.

## The pattern

To save bytes, a cell stores the first twenty bytes of an owner's lock hash. A guard is
written to refuse an owner set to the cell's own always-success lock (which would make the
cell public property):

```rust
if owner_lock_hash == cell_lock_hash { return Err(OwnerIsCellLock) }
```

`owner_lock_hash` is twenty bytes. `cell_lock_hash` is thirty-two. A whole-slice compare
is never equal, so the guard **never fires**, and a cell created with its owner set to
the always-success prefix was accepted. Because the ownership check elsewhere compared
prefixes correctly, that cell was then editable and transferable by anyone who touched
any cell of the type.

The same shape, the other way round: a third party's twenty-byte identity compared
against a thirty-two-byte constant naming the fee recipient, so the recipient could pose
as that third party and be paid a cut of its own fee.

The same class through molecule. A `packed::Bytes` (a witness's `lock`, any `Bytes` field)
answers `as_slice()` with its four length bytes in front, and `raw_data()` without them. A
check for "is this witness still the empty placeholder, 65 zero bytes" written over
`as_slice()` compares 69 bytes with 65 and is never true. In a node's channel funding code
that kept a peer's zero placeholder over a real signature, and some funding transactions
failed signature verification at random ([nervosnetwork/fiber#1715](https://github.com/nervosnetwork/fiber/pull/1715),
fixed 2026-10-29). In a script the same compare under a guard that refuses on equality
never refuses.

## The question to ask

For every comparison in the script: **what length is each side, and what happens if they
differ?** Then: if this check fails by mistake, is the result to refuse (safe) or to
accept (a hole)? A comparison whose false branch is the dangerous one must be proven to
be reachable.

## The fix

Compare the prefix explicitly, and say so:

```rust
if hash == &cell_lock[..OWNER_HASH_LEN] { return Err(OwnerIsCellLock) }
```

Through molecule, compare what the field carries, or two values of the same type:

```rust
if witness.lock().to_opt().map(|l| l.raw_data()).as_deref() == Some(&PLACEHOLDER[..]) { … }
```

## The test

A test that sets the forbidden value and expects the error. The one that existed set the
forbidden value **and** the matching cell lock in the same fixture, and passed for the
wrong reason; it never varied the thing the guard compares. Vary one side at a time.

## Runnable

`code/contracts/owned-type`, test `p03_a_truncated_compare_fails_open`, which also shows
the consequence: a stranger editing the cell the guard should have refused. The molecule
case: `p03_molecule_bytes_keep_their_length`, on the host.
