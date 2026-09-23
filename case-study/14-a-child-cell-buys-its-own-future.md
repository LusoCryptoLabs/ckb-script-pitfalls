# 14. A child cell buys its own future

**Class:** a permissionless action that is right for one kind of cell and wrong for a
dependent kind. **Severity:** Medium: a holder keeps a sub-name alive through the parent's
change of hands, and the new owner cannot evict it.

## The pattern

Renewal is permissionless by design: anyone may pay to extend anyone's name, which means
a name cannot be lost because its owner was unreachable, and paying gives no claim. The
rule was written for plain names and applied unchanged to **sub-names** (`shop.brand`),
which the parent's owner must sign to create.

Creation asked for the parent owner's signature. Renewal did not. So the holder of
`shop.brand` could renew `brand` for its owner (anyone may), then `shop.brand` for
themselves, and keep the child alive through a sale of the parent, or through its lapse,
recycle and fresh registration by a stranger. The rule a person assumes, that a name under
mine lives at my pleasure, held at creation and at no moment after.

It had been written down as a "design note" rather than a finding, with the reasoning that
requiring the parent's signature at renewal "costs more than it buys". Three days later
the owner of the protocol set the rule in one sentence and it was judged the cheaper one.

## The fix

One line in the renewal validator's sub-name branch, the same proof creation asks for:

```rust
if let Some(parent) = parent_label(label) {
    let p = require_parent(parent)?;           // a cell dep of this type with that label
    if child.expired_at() > p.expired_at() { return Err(ParentOutlived) }
    require_owner(p.owner_lock_hash())?;       // an input under the parent owner's lock
}
```

A plain name's renewal stays permissionless. Only the dotted label asks for a signature.

## The question to ask

For every permissionless action: **is there a kind of cell for which "anyone" should
mean "one particular person"?** Dependent cells (children, delegates, listings) usually
have one.

## The test

Reproduced against the deployed binary first: renewal by nobody, and renewal by the child's
own owner without the parent's, both reported "expected a failure, it passed". After the
fix both refuse with the authorization error; the control (the parent's owner renewing)
passes, and the plain-name renewal with nobody signing is unchanged. Then on the live
network: the old transaction shape refused by the node with the script's error code, the
new one landed and read back a year longer.

## Where it lives

The contracts are public at [cells-contracts](https://github.com/LusoCryptoLabs/cells-contracts). The tests that pin it, in [`tests/tests/account_cell.rs`](https://github.com/LusoCryptoLabs/cells-contracts/blob/main/tests/tests/account_cell.rs): `subname_renew_by_nobody_is_refused` and `subname_renew_by_the_child_owner_alone_is_refused`, with `subname_renew_by_the_parent_owner_passes` as the control.
