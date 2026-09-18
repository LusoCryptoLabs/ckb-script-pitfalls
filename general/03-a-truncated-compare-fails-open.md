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

## The test

A test that sets the forbidden value and expects the error. The one that existed set the
forbidden value **and** the matching cell lock in the same fixture, and passed for the
wrong reason; it never varied the thing the guard compares. Vary one side at a time.

## Runnable

`code/contracts/owned-type`, test `p03_a_truncated_compare_fails_open`, which also shows
the consequence: a stranger editing the cell the guard should have refused.
