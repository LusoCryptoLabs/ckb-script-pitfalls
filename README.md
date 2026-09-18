# CKB script pitfalls

Ways a Nervos CKB script goes wrong that have each cost somebody something, written
down so the next person does not pay for them again.

Every entry here was found in a script running on a public network, reproduced in the
VM against the deployed binary before it was fixed, and fixed with a test that fails on
the old code and passes on the new one, with a control beside it. None of them is a
theory. Most of them survived several review passes and a growing suite of example
tests before somebody asked the right question.

They are specific to how CKB works: a type script runs on inputs and on outputs, it
sees the whole transaction and nothing outside it, every instance in a transaction
reads the same outputs, capacity is money, and a lock that always succeeds is a
legitimate design. Solidity habits do not cover these.

## The rule that makes a review worth something

**Reproduce the hole against the script as it stands before you touch it.** Write the
test that describes the abuse, run it against the current binary, and watch it report
that the transaction was accepted. Then fix, and watch it fail. Then run the control: the
honest transaction of the same shape still passes.

Without that you deliver a plausible story and the other side has to believe you. With
it you deliver a fact. A test that only ever ran green proves nothing about what it
guards; several of the entries below were guarded by tests that could not fail.

## The catalogue

Numbered in the order they were found. The gaps are entries that only make sense against
one particular script's structure and are not in this branch.

| # | Pitfall | The one-line version |
|---|---|---|
| [01](general/01-a-payment-sum-is-safe-only-against-itself.md) | A payment sum is safe only against itself | "Sum the outputs paying lock L and require at least X" is satisfied once for every script that asks |
| [02](general/02-a-typed-output-meets-the-capacity.md) | A typed output meets the capacity | A payment output with a hostile type script has the right capacity and cannot be spent |
| [03](general/03-a-truncated-compare-fails-open.md) | A truncated compare fails open | Twenty bytes never equal thirty-two, so the guard admits everything |
| [07](general/07-capacity-is-unguarded-on-permissionless-actions.md) | Capacity is unguarded on permissionless actions | Pin every byte of data and leave the capacity free, and a stranger takes the rent |
| [09](general/09-since-flags.md) | `since` flags | The metric and the relative bit live in the top byte; 0x40 is absolute time, 0xC0 relative |
| [10](general/10-witnesses-by-output-index.md) | Witnesses by output index | An output's witness is read through `Source::Input` indexing, and the order must be pinned |
| [11](general/11-queryiter-panics.md) | `QueryIter` panics | Fail-closed, but with the library's exit code, not yours |
| [12](general/12-comments-are-the-specification.md) | Comments are the specification | A comment that promises more than the code does is a defect; measure every "roughly" |
| [13](general/13-examples-are-not-properties.md) | Examples are not properties | 171 example tests missed a hole a 225-point grid found in one run |

## Run them

[`code/`](code/README.md) holds a toy script per general entry, built twice, as first
written and with the correction, and a test per entry that runs the exploiting transaction
against both: accepted by the one, refused by the other with the named error code, and
the honest transaction accepted by both. `cd code && make test`.

## The review method

[`skill/ckb-script-review/SKILL.md`](skill/ckb-script-review/SKILL.md) is the method
that produced the catalogue, written as a skill for Claude Code: the questions to ask
of any CKB script, in the order they find things, and the discipline of proof above. It
is plain Markdown and reads as a checklist without the tool.

## Contributing

An entry needs three things: the pattern, the transaction that exploits it, and the test
that catches it with its control. A concern without a transaction is a question, and
questions are welcome as issues.

MIT.
