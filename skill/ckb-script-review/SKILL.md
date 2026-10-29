---
name: ckb-script-review
description: Review a Nervos CKB script (a Rust type or lock script) with scepticism, looking for real holes. Use when asked to audit, review, sanity-check or "have a look at" CKB contracts, your own or a dependency's. Carries the questions that have found real bugs, the steps to run (mutation, the readers on the host, fuzzing with an oracle, attacks on the deployed script), and the discipline of proving a hole before closing it.
---

# Reviewing a CKB script

What makes this review worth something is not knowing Rust. It is a handful of questions
specific to CKB, each of which has already cost somebody something, and one rule of proof.
The catalogue of what those questions found is in the `general/` directory beside this file; each entry has the pattern, the exploiting transaction and the test.

## The rule that separates a review from an opinion

**Reproduce the hole against the script as it stands before you touch it.** A test that
fails on the current binary, passes with the fix, and a control beside it proving the new
rule refuses the abuse and not the honest case.

Without that you deliver a plausible story and the other side has to believe you. With it
you deliver a fact. The order, always:

1. write the test that describes the abuse;
2. run it against the script **without** the fix: it must fail ("expected a failure, it
   passed");
3. apply the fix: it must pass;
4. the control: the honest transaction still passes.

Then, if it is deployed, the same two transactions against the live network. A green suite
does not prove the deployed binary is the one you reviewed; a reproducible-build check
does.

## The questions that find things

### 1. Per script group, and in both directions

A type script runs once per group, on inputs **and** outputs. Nearly every bug lives
there, not on the happy path:

- what if the cell appears only in inputs (a burn, a recycle)?
- what if it appears only in outputs (creation)?
- what if several of the same type appear in one transaction?
- does the validator pin the arity, or assume it?

### 2. Which comparisons can fail OPEN

A stored identity truncated to twenty bytes compared with a thirty-two byte hash is never
equal, so the guard admits everything instead of refusing everything. For every
comparison: **what length is each side, and what if they differ?** Then: if this check
fails by mistake, is the result to refuse (safe) or to accept (a hole)?

Two more shapes of the same question. A molecule `Bytes` answers `as_slice()` with its
four length bytes in front; compared with the bytes it carries it is never equal, so
compare `raw_data()`. And two reads that can both fail: `a() != b()` over `Option`s lets
`None == None` through, so turn each failure into an error before comparing.

### 3. Which sums are shared between instances

When several instances of a script run in one transaction, **all read the same outputs**.
Each seeing enough for itself is not the same as the total being paid. Two cells of one
payee; two payees and a shared fee leg; two *different* scripts owing the same
treasury; two deployments of one script sharing a fee lock. A check of the form "sum what is paid to L" is
safe only against instances of the same script. If the answer is "the arity makes that
unreachable", ask: **and with a second deployment?**

### 4. Whose money is each input, and where may it go

Capacity is money. For every action that consumes a cell of the type and produces a
successor: is the capacity pinned not to shrink? The same for a lock that lets strangers
spend its cell (a pool, a claim): its successor's capacity is theirs to take unless the
lock says otherwise. For every input the script unlocks (a
deposit parked in a cell): does the validator let it flow to whoever built the transaction?
Take a transaction that was just called a success and ask where each side's money came
from.

### 5. What decides an output's lock and type

For every cell the script creates: who chose its lock, and what stops them choosing one
nobody can spend, or one that makes the cell's own capacity count as a payment? For every
payment output: is a type script allowed on it? A hostile type meets the capacity and
cannot be spent.

### 6. What the exemptions admit as written

"The id is zero" is not "the one cell that id was reserved for". "Past its date" includes
a cell created past it. "Anyone may extend" applied to a dependent cell lets it outlive
the cell it depends on changing hands. For every structural exemption and every
permissionless action: what else satisfies the condition, and is there a kind of cell for
which "anyone" should be one person?

### 7. Which locks open for someone who is not the owner

A lock that defers to the protocol's type, an anyone-can-pay lock, an always-success
instance: each opens for whoever builds a valid transaction, and its hash is still an
identity. UDT owner mode asks only for an input under the owner lock, so a token naming
such a lock is minted by anyone who can spend a cell under it. For every lock the protocol
defines or accepts: who can open it, and what names its hash? The protocol's own LP is
fine only because the type pins its supply.

### 8. What panics instead of returning an error

`QueryIter` in `ckb-std` panics on any syscall error but `IndexOutOfBound` (read the
version the project pins). Fail-closed, but with the library's exit code, so diagnosis
costs time. Note a file that mixes idioms.

### 9. Which assumption is undocumented

Reading witnesses by output index through `Source::Input` works (it is a plain list
lookup) but must be pinned by the validator's own ordering. Mark such things medium
confidence and say how to verify (the RFC, or a case with fewer inputs than outputs).

### 10. Which reads can come back cut

A read syscall with a buffer smaller than the data returns `LengthNotEnough` and fills the
buffer anyway. A script that treats that as success, or reads into a fixed buffer without
looking at the real length, accepts truncated data as whole. For every read: how big is the
buffer, and what happens to the rest? Found in iCKB by ScaleBit (2024, UTI-1,
informational).

### 11. Which cells someone else can imitate

Output locks do not run. Anyone creates a cell with the same type, the same data and the
look of somebody else's cell, and that cell's lock has no say. If the product finds "its"
cells by shape and not by origin, an imitation passes for the real one; seen in iCKB's limit
orders (ScaleBit, ENT1-1). For every cell the product looks for: how does it know it made
it? Following a cell from a known genesis out point is one answer.

### 12. Heap and cycles

`default_alloc!()` gives a fixed heap. A walk that `load_cell_data`s **every** input to
find thirty-two bytes allocates each. On a permissionless path that is a grief lever.
Prefer loading a bounded prefix and discarding by length.

### 13. The comments are the specification

A comment that promises more than the code does is a defect, even when the code is the
safer of the two. Every "roughly" is a number to measure once.

## Verify instead of recall

Read the source of the `ckb-std` and `ckb-script` versions the project actually uses,
not the ones you remember. Confirm constants against the RFCs: `since` has bit 63
relative and bits 62 and 61 metric, so an absolute timestamp is `0x40` in the top byte and
a relative one `0xC0`; header timestamps are milliseconds, `since` timestamps seconds.
A `since` compared through `ckb_std::since::Since` is not ordered when the flags differ:
`lt` and `ge` are both false, so a check must refuse on `!ge`, never pass on `!lt`.

## The steps to run

The questions say where to look. These say what to run, so that none of it depends on
somebody remembering.

### 1. Mutation: every guard has a test that catches it

Switch each check off, one at a time, rebuild, and run the test named for it: it must
fail. A guard whose test stays green is not tested, however green the suite is.
`cargo-mutants` fits badly because the tests load the compiled RISC-V binary; a short
script does it, a list of (guard, file, original text, mutated text, test) applied to a
copy, with KILLED or SURVIVED per guard. Each new guard enters the list in the commit
that adds it.

### 2. The readers on the host

The functions that read bytes (args, witness, data layouts, a proof's framing) can be
pure. Move them out of the entry point into a `no_std` library without syscalls, which the
script uses and the host compiles. Then:

- property tests (`proptest`, stable): arbitrary bytes never panic, and what is accepted
  is read once, meaning it encodes back to exactly the same bytes; a witness built to be
  read comes back equal, and one byte more or less is refused; built ones with bytes
  changed or cut stay read once or refused;
- the same properties under libFuzzer (`cargo fuzz run -O -a`, nightly), seeded with real
  witnesses the chain accepted; `-a` is safe here because `ckb-std` is not in the build;
- a differential where a reference exists: a one-leaf transaction proof against
  `ckb-types`' CBMT on random blocks.

The move changes the binary (one script grew by about 2.5 KB, because it now reads the
whole witness before checking it), and an immutable lock's new binary is a new
deployment. Decide it before deploying.

### 3. Proofs the builder writes: fuzz with an oracle

When the script verifies a proof that the transaction's builder supplies (a sparse merkle
tree's compiled proof, which is a small opcode program), crash-fuzzing misses the failure
that matters: a forged proof that verifies. Fuzz it against a known tree with an oracle:
whenever a proof verifies against the tree's root, every value it proved is the tree's.
Pick keys present and absent, values true, zero or wrong, then edit the honest proof (a
byte changed, cut, inserted, deleted) or bring raw bytes. Run the control first: with the
verifier replaced by one that accepts everything, the oracle must fail within seconds.

### 4. The whole script under a fuzzer, and when it pays

The Nervos `ckb-script-fuzzing-toolkit` compiles the script for x86_64 with simulated
syscalls; seeds come from the tests (`dump_tx` in ckb-testtool, converted by
`ckb-vm-syscall-tracer`), under libFuzzer, Honggfuzz or AFL++ with sanitizers. Guide:
docs.nervos.org/docs/script/fuzzing-ckb-scripts. It needs a `ckb-std` with the native
simulator (1.1). It finds crashes. In a lock a crash refuses the transaction, and the
fuzzer cannot tell a wrong acceptance from a right one, so for a lock it buys less than
steps 2 and 3; it pays where a crash could block honest users. When you run it:

- the script must be a library (`program_entry` in `lib.rs`), `main.rs` only the VM entry;
- `cargo fuzz run -O`: without `-O`, debug assertions inside `ckb-std` stop on cases the
  real VM never reaches;
- replay each seed in the harness first and check the exit codes the tests expect;
- a build without the signature library refuses every signature: it covers parsing and
  dispatch, not what follows a valid signature.

### 5. Builder and reader agree

The code that builds the transaction and the script that reads it run on the same fixed
inputs and compare byte for byte: config, witness, the signed message. Personalization
and length framing are where they drift.

### 6. Reproducible build

Rebuild the reviewed commit with the pinned toolchain, hash the binary and compare with
the code cell's data hash on chain. If they differ, the code reviewed is not the code
running. For anyone else to reach the same bytes, `ckb-script-templates` has
`scripts/reproducible_build_docker`, Rust and LLVM pinned in a container and a
`checksums.txt` checked in CI.

### 7. Dependencies and outside binaries

`cargo audit` on the lock file the script's binary is built from, not only the workspace's
(test crates bring advisories that never reach the chain); `cargo tree -i <crate>` says who
pulls each one. For every C binary loaded by data hash: which commit, which compiler, can
someone else rebuild the bytes, and which fixes landed later in the files it is built from.

### 8. The largest real case

Every limit, quorum and size at the largest value the product can produce, with cycles
measured (`verify_tx` returns them) against the node's per-transaction limit (70 M by
default), not the harness's. A case the product lets someone create that does not fit is
money stuck. Size too: a witness that carries other transactions whole grows with them.
`ckb-debugger --pprof` shows where a path's cycles go.

### 9. Attacks on the deployed script, spending nothing

`test_tx_pool_accept` runs the node's full verifier without sending; `estimate_cycles`
runs scripts only and does not check capacity. Each forged transaction beside the honest
one, which passes. The node checks two things before any script, and a forgery that trips
on them says nothing about the guard: every output at least its occupied capacity (taking
CKB from a cell already at its minimum is refused there, so test with spare capacity), and
the minimum fee rate for the size (bytes added to a witness after the fee was set fall
short by a shannon).

### 10. Coverage

Which lines no test crosses. `ckb-script-templates` has `make coverage` through the
native simulator. A check no test reaches is a guard without proof.

### 11. Proofs over the readers

Kani model-checks Rust: a harness with `kani::any()` covers every input up to a size.
Candidates: the readers of step 2, proven never to panic and never to read out of bounds.

### 12. State machines against a model

Where the script has states, `proptest-state-machine` generates step sequences, applies
them to a simple model and to the script (each step a ckb-testtool transaction), compares
accept or refuse at each step and shrinks a failure to the smallest case.

### 13. Conformance vectors for cryptography

A signature library the script loads is tested with public vectors, not only your own
signatures. For ECDSA P-256 with SHA-256, Wycheproof's `ecdsa_secp256r1_sha256_test.json`
(C2SP, `testvectors_v1`), the "acceptable" cases handled one by one.

### 14. Replay history before an upgrade

Before replacing a binary, run the real transactions the old one validated against the new
one with `ckb-debugger`: what passed still passes, and every state cell that exists still
reads.

### 15. Static analysis

Clippy's restriction lints mark where an index, an arithmetic operation or a cast can fail:

```
cargo clippy --release -- -W clippy::indexing_slicing -W clippy::arithmetic_side_effects \
  -W clippy::unwrap_used -W clippy::panic -W clippy::cast_possible_truncation
```

They are not findings: follow each to the value that feeds it; most are guarded by a check
before. Then `cargo-geiger` (unsafe from dependencies) and `cargo-deny` (git dependencies,
licences and advisories as a CI rule).

## Writing the review

1. **Summary**: what the script tries to do in the cell model.
2. **Findings**, each with the exploiting transaction and the test that catches it, with
   its control.
3. **Cross-check**: the Rust Book for semantics, the Nervos docs and `ckb-std` for the
   rest.
4. **Suggested fixes**: minimal, with the trade-off stated. A security fix does not travel
   with idiom or performance improvements; those wait for the next change.
5. **Confidence** per point, high, medium or low, and how to raise the low ones.

Filter against the project's own security log first: re-reporting a closed finding is
noise. Say also what was **not** reviewed.

## Before opening the code

A disclosure policy in the repository: where to report, and how fast you answer.

## After a fix that ships

A type-id upgrade keeps the code hash, so existing cells keep validating; the dep outpoint
moves and every mirror of it (services, apps, watchers) must be repointed. Then a
real transaction: a suite proves nothing about the deployed guardian accepting honest
work. And check that whatever watches the code cell **actually alerts**: a watcher that
has never been seen to fire is a watcher nobody knows is connected.
