---
name: ckb-script-review
description: Review a Nervos CKB script (a Rust type or lock script) with scepticism, looking for real holes. Use when asked to audit, review, sanity-check or "have a look at" CKB contracts, your own or a dependency's. Carries the questions that have found real bugs and the discipline of proving a hole before closing it.
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

### 3. Which sums are shared between instances

When several instances of a script run in one transaction, **all read the same outputs**.
Each seeing enough for itself is not the same as the total being paid. Two cells of one
payee; two payees and a shared fee leg; two *different* scripts owing the same
treasury; two deployments of one script sharing a fee lock. A check of the form "sum what is paid to L" is
safe only against instances of the same script. If the answer is "the arity makes that
unreachable", ask: **and with a second deployment?**

### 4. Whose money is each input, and where may it go

Capacity is money. For every action that consumes a cell of the type and produces a
successor: is the capacity pinned not to shrink? For every input the script unlocks (a
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

### 7. What panics instead of returning an error

`QueryIter` in `ckb-std` panics on any syscall error but `IndexOutOfBound` (read the
version the project pins). Fail-closed, but with the library's exit code, so diagnosis
costs time. Note a file that mixes idioms.

### 8. Which assumption is undocumented

Reading witnesses by output index through `Source::Input` works (it is a plain list
lookup) but must be pinned by the validator's own ordering. Mark such things medium
confidence and say how to verify (the RFC, or a case with fewer inputs than outputs).

### 9. Heap and cycles

`default_alloc!()` gives a fixed heap. A walk that `load_cell_data`s **every** input to
find thirty-two bytes allocates each. On a permissionless path that is a grief lever.
Prefer loading a bounded prefix and discarding by length.

### 10. The comments are the specification

A comment that promises more than the code does is a defect, even when the code is the
safer of the two. Every "roughly" is a number to measure once.

## Verify instead of recall

Read the source of the `ckb-std` and `ckb-script` versions the project actually uses,
not the ones you remember. Confirm constants against the RFCs: `since` has bit 63
relative and bits 62 and 61 metric, so an absolute timestamp is `0x40` in the top byte and
a relative one `0xC0`; header timestamps are milliseconds, `since` timestamps seconds.

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

## After a fix that ships

A type-id upgrade keeps the code hash, so existing cells keep validating; the dep outpoint
moves and every mirror of it (services, apps, watchers) must be repointed. Then a
real transaction: a suite proves nothing about the deployed guardian accepting honest
work. And check that whatever watches the code cell **actually alerts**: a watcher that
has never been seen to fire is a watcher nobody knows is connected.
