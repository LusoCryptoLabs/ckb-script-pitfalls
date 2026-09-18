# 07. Capacity is unguarded on permissionless actions

**Class:** every byte of data pinned, the money left free. **Severity:** High: a
stranger takes the refundable rent of any cell they are allowed to touch.

## The pattern

A cell's capacity is its owner's money: the refundable rent that keeps it stored. A type
script with an always-success lock lets **anyone** build certain transactions on a cell
(update it, clear a spent neighbour, insert beside it), and keeps the owner safe by
pinning the data: every field byte-equal between input and output.

Capacity is not data. Five actions pinned every field and left the output's capacity free.
Three of those were permissionless. So a stranger sets the surviving output's capacity to
the occupied minimum and keeps the difference: the cell survives, the deposit does not.

Only the insertion action had the check, on the neighbour it spends, because that path was
written with "a stranger spends someone else's cell" in mind. The others were written as
"the same cell continues" and nobody asked what "the same" meant for the one field that
is money.

## The fix

One helper, called at every site where a cell of the type survives an action:

```rust
fn capacity_not_reduced(input: &Cell, output: &Cell) -> Result<(), Error> {
    if load_cell_capacity(output.index, Source::Output)? < load_cell_capacity(input.index, Source::Input)? {
        return Err(StructuralDrift)
    }
    Ok(())
}
```

Increasing is fine (somebody topping up a cell harms nobody).

## The question to ask

For every action that consumes a cell of the type and produces its successor: **where
does the capacity go?** List the actions a stranger may perform and ask it of each.

## The test

For each action, the honest transaction with the output's capacity one shannon below the
input's: refused. Then a property over a grid of capacities: the script accepts exactly
when `out >= in`. The property found the same thing in one run that five example tests
had to be written for one by one.

## Runnable

`code/contracts/owned-type`, test `p07_capacity_is_unguarded_on_permissionless_actions`.

## References

- Capacity as the measure of state, and why it is money: [RFC 0002, the cell model](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0002-ckb/0002-ckb.md)
