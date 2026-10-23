# 15. A lock anyone can open is an owner key

**Class:** authority that leaks through a lock hash. **Severity:** High wherever value
sits beside a token that names such a lock: whoever can open the lock mints the token.

## The pattern

Many CKB protocols give their cells a lock that does not judge. It opens whenever the
protocol's type script is in the transaction and leaves every decision to that type. For
the protocol's own cells this is sound: the type runs, and the type decides.

But a lock hash is also an identity. sUDT and xUDT name their owner by a lock hash, and
owner mode, which lifts the rule that outputs may not exceed inputs, asks only for an
input under that lock. xUDT flags can add type hashes as owners, which widens the same
door. So the hash of a lock that anyone can open, given a valid protocol transaction, is a
mint key that anyone can use.

The protocol's own LP token is the legitimate case: it names the lock on purpose, and the
type pins the LP supply to exactly what each action earns. Every other token that names
the lock has no such guard. Nobody holds its owner key, so it reads as renounced. In fact
everybody holds it.

Found in a constant-product AMM's published source before it was deployed, and reproduced
in the VM against a build of that source with the mainnet xUDT binary. The pool reserves
sat under such a lock. A token T named the reserve lock of an honest pool Q as its owner.
A one CKB swap on Q, with 10^24 T added to the outputs, was accepted; that T sold into a
CKB/T pool took its reserve from 5,000 CKB to one shannon. The pool's type script used as
a lock opened the same way.

## The fix

In the protocol's type script, which runs in every transaction that can open the lock:
refuse any UDT whose supply rises, except the tokens the protocol mints itself and whose
supply it already pins.

```rust
for each output with an xUDT type, once per type hash:
    if type != own_lp && supply(type, Source::Output) > supply(type, Source::Input) {
        return Err(ForeignMint)
    }
```

Cover every UDT family in use (xUDT in all its args forms, sUDT), not only the ones the
protocol trades: a token minted here can be sold anywhere. The cost is a walk over the
outputs; in the AMM it added 21 to 33% to transactions of 0.5 to 1.5 million cycles.

This closes the protocol's own locks. A token naming some other lock that anyone can open
(an anyone-can-pay lock, an always-success instance) stays a vetting problem: a token
checker should resolve the owner hash before it calls a token renounced.

## The question to ask

**Which of the protocol's locks open for someone who is not the cell's owner, and what
names their hash?** UDT owner fields first. Then anything that takes "an input under lock
L" as proof of authority.

## The test

A permissionless action on the protocol (here, topping up a vault) with one more output:
a large amount of a token whose owner is the hash of the protocol's lock. Accepted as first
written, refused with the fix. Two controls: the action without the mint passes on both
builds, and the same mint naming a lock that is not in the transaction fails on both,
which shows it was the protocol's lock that opened owner mode.

## Runnable

`code/contracts/delegate-vault`, test `p15_a_lock_anyone_can_open_is_an_owner_key`, run
against the mainnet xUDT binary in `code/tests/bin/xudt`.

## References

- xUDT and its owner mode: [RFC 0052, extensible UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0052-extensible-udt/0052-extensible-udt.md)
- sUDT: [RFC 0025, simple UDT](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md)
