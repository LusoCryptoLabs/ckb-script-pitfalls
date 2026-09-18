# 11. `QueryIter` panics

**Class:** a fail-closed behaviour that hides the reason. **Severity:** low, but it costs
hours in diagnosis and it decides how your error codes read to the outside.

## The facts

`ckb_std::high_level::QueryIter` calls `panic!` on any syscall error that is not
`IndexOutOfBound` (checked in `ckb-std` 0.15.3, `high_level.rs`). A panic exits the VM with
the library's code, not one of your protocol's error codes. The transaction is refused,
which is safe, but the explorer page that maps a script's error codes to names shows
nothing useful, and the person building the transaction cannot tell a malformed cell from
a bug in your script.

## The pattern

A file that mixes idioms: `QueryIter` in one function, an explicit loop with typed errors
in the next.

```rust
// panics on anything but IndexOutOfBound
QueryIter::new(load_cell_lock_hash, Source::Input).any(|h| h[..20] == *owner)

// returns your error
let mut i = 0;
loop {
    match load_cell_lock_hash(i, Source::Input) {
        Ok(h) => { if h[..20] == *owner { return Ok(()) } i += 1 }
        Err(SysError::IndexOutOfBound) => return Err(Error::Unauthorized),
        Err(_) => return Err(Error::Encoding),
    }
}
```

Both are fail-closed. Only the second tells anyone why.

## What to do

Pick one idiom for the script and say which. If `QueryIter` stays, know that every
walk using it fails with the library's code, and keep it away from the paths where a
caller needs to learn the reason (the ones that take user input). Read the version of
`ckb-std` the project actually pins; behaviour has changed between versions.

## References

- `QueryIter` in `ckb-std`, `src/high_level.rs`: read the version your project pins.
