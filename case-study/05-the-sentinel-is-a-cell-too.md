# 05. The sentinel is a cell too

**Class:** an exemption written wider than the thing it exempts. **Severity:** Critical:
uniqueness broken.

## The pattern

A sorted circular list of cells enforces that a name is registered once: each cell covers
the id range up to its successor, and inserting `x` means spending the unique cell whose
range holds `x`. The list needs a root: a sentinel cell with id zero, no label, created at
genesis with `expired_at = 0`.

Recycling an expired cell is permissionless. The recycle validator checked that the
target was expired (the root always is) and that the predecessor covered it (the last
node's range is `(max, 0]`, and zero is the root). Nothing said the root is not a name.

With the root gone the ring covers id zero. The integrity check exempted "any cell whose
id is zero" from the label-to-id derivation, because the root has no label. So the next
registration could mint a cell with id zero and **any label, including one already
registered**. Two live cells, same name.

Three of four independent reviewers found it. The property test cited as evidence for the
invariant had **excluded the root from recycling and id zero from registration**: it
hard-coded the two assumptions the contract failed to enforce, and 8,000 random operations
could never reach the hole.

## The fix

Two lines that each close it alone, so that neither depends on the other:

- `recycle` refuses the root's id;
- `register` refuses a new cell at the root's id;

and the integrity exemption narrowed from "id is zero" to "id is zero **and** the label is
empty".

## The question to ask

Every structural exemption: **what else satisfies the condition as written?** "Id is zero"
is not "the root"; "expired" includes a cell created expired on purpose.

## The test

Recycle the root: refused with the root error. Control: the same transaction shape against
an ordinary live cell is refused with **not expired**, which shows the transaction was
well formed and the first refusal was the rule, not a build error. Proven on the live
network the same way.

## Where it lives

The contracts are public at [cells-contracts](https://github.com/LusoCryptoLabs/cells-contracts). The tests that pin it, in [`tests/tests/account_cell.rs`](https://github.com/LusoCryptoLabs/cells-contracts/blob/main/tests/tests/account_cell.rs): `recycle_must_not_remove_the_root`, and `register_refuses_the_empty_label_even_where_the_ring_admits_id_zero` for the other half of the same hole.
