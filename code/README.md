# Runnable pitfalls

Each general entry in the catalogue has a toy script here that exhibits exactly one
mistake, built twice: as first written (`target/vulnerable/`) and with the correction
(`target/fixed/`, the `fixed` cargo feature). The tests in `tests/tests/pitfalls.rs` run
the same transaction against both and assert what the entry claims: accepted by the one,
refused by the other with the named error code, and the honest transaction of the same
shape accepted by both.

| toy | pitfalls | what it is |
|---|---|---|
| `fee-type` | 01, 02, 13 | a type script whose cells owe a fee to a lock named in the args when spent |
| `owned-type` | 03, 07 | a cell with an owner identity in its data, an always-success lock, and one permissionless action |
| `timed-lock` | 09 | a lock that opens once the input's `since` proves a date has passed |
| `payload-type` | 10 | a cell whose data commits to bytes carried in the witness at the cell's output index |

## Run

Needs the Rust toolchain with the `riscv64imac-unknown-none-elf` target (the
`rust-toolchain.toml` here adds it), `make`, and a C compiler that targets RISC-V for
the small shim `ckb-std` builds: `clang` and `llvm` from your package manager are enough
(the Makefile points `cc` at them when `riscv64-unknown-elf-gcc` is absent). `ckb-testtool`
brings the VM. Proven from zero in `docker run rust:1.85-bookworm` with
`apt-get install make clang llvm`.

```sh
cd code
make test
```

`make build` alone leaves both builds under `target/`. The tests find them through
`PITFALLS_TARGET` if built elsewhere.

## Reading a test

```rust
let attack = |b| fee_tx(b, &[100, 50], &[Pay { ckb: 100, typed: false }]);
passed(&attack(Vulnerable), "01 vulnerable: one output of 100 answers fees of 100 and 50");
refused(&attack(Fixed), 3, "01 fixed: the total of 150 is demanded");
```

`refused` checks the error **code**, not merely that the transaction failed: a test that
fails for a different reason than the one under test is a test that proves nothing about
it, and that mistake is in the catalogue too (13).

## Adding one

A toy script that does one thing, a `fixed` feature that closes one hole, and a test in
the shape above. If the mistake needs two scripts in one transaction to show, deploy both
in the same `Context`; several of the entries were missed for months because no test
file ever did.
