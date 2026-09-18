# 10. Witnesses by output index

**Class:** where a type script reads its payload from, and what pins it there.
**Severity:** a payload read from the wrong slot, or an action read from a slot the
attacker controls.

## The facts

`load_witness_args(i, Source::Input)` resolves to plain `witnesses.get(i)` (checked in
`ckb-script` 0.119). There is no "witness of output i"; the convention is that an output
cell's payload lives in `witnesses[i].output_type` where `i` is the **output's index**,
and the same slot's `input_type` is free for the input at that index. A cell may be both
an input and a cell dep in one transaction.

## Where it goes wrong

- Reading `witnesses[0].input_type` for the action while the attacker can reorder outputs
  so that the cell of interest is not at index 0. If the validator pins the order (the
  surviving cell is `outputs[0]` and must carry the spent cell's key, the new one is
  `outputs[1]`), reordering fails elsewhere and the read is safe. If nothing pins it, the
  read is attacker-chosen.
- Reading a payload through `Source::Output` and expecting it to differ.
- A transaction with fewer inputs than outputs: the slot exists (witnesses are a plain
  list), but a client that sizes witnesses by input count leaves it empty. Pin the
  client's output order and pad the witness list before injecting payloads.

## What keeps it fail-closed

Every output of the type must have its payload hash to a commitment stored in its data.
The script recomputes `blake2b(witnesses[i].output_type)` and compares; a missing or wrong
slot is a mismatch and the transaction fails. The order of outputs is fixed by the
validator's own arithmetic on the cells' ids.

## The test

The payload read by output index past the input count (more outputs than inputs): passes
when the client padded the list, and the same transaction with the payload moved one slot
over fails with the commitment mismatch.

## Runnable

`code/contracts/payload-type`, test `p10_witnesses_by_output_index`.

## References

- `WitnessArgs` and the witness list: [RFC 0022](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0022-transaction-structure/0022-transaction-structure.md)
- `load_witness_args` resolving to the list by index: `ckb-script`, `TransactionScriptsVerifier`
