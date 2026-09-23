# 06. A commitment is public the moment it lands

**Class:** commit-reveal that binds the secret but not the committer. **Severity:**
Critical: the registration can be front-run, and a second gap let the front-runner choose
fields the commitment never covered.

## The pattern

To stop mempool watchers stealing a name, registration is two steps: commit a hash of
`namespace ‖ label ‖ owner ‖ secret` in a cell, wait a minimum delay (a relative
`since`), then reveal. The reveal validator finds the commit cell by matching its data to
the recomputed hash.

A commitment is the cell's entire data. It is **public the moment the commit confirms**, a
full delay before the reveal. Anyone copies those thirty-two bytes into a cell of their
own, matures it on the same clock, reads the secret out of the victim's broadcast reveal
and lands the registration first. The name still goes to the committed owner, but the
front-runner chose everything else about it.

Everything else included the **manager**, a delegate field the commitment did not cover
and the register validator did not constrain. Chained together: permanent control of a
name's records, on a name that reads as its owner's.

## The fix

- The matching commit cell must be **held by the owner it commits to**: its lock hash
  prefix equals the new cell's owner. A copied commitment is then worthless, because the
  copier cannot produce a cell under the victim's lock. Honest clients already paid the
  commit cell to themselves, so this costs them nothing.
- Every field the commitment does not cover is fixed by the validator at registration
  (here: manager must equal owner; delegation is a separate, owner-signed action).

## The question to ask

For a commit-reveal: **who can produce a cell with this data?** Anyone, unless the lock is
part of the check. And: **which fields of the revealed cell does the commitment not
cover, and who chooses them?**

## The test

A reveal whose matching commit cell sits under a stranger's lock: refused with
"commit not owned". Control: the same reveal with the commit under the owner's lock passes.
And a registration that sets a manager different from the owner: refused.

## Where it lives

The contracts are public at [cells-contracts](https://github.com/LusoCryptoLabs/cells-contracts). The tests that pin it, in [`tests/tests/account_cell.rs`](https://github.com/LusoCryptoLabs/cells-contracts/blob/main/tests/tests/account_cell.rs): `register_rejects_a_commit_held_by_a_stranger` for the ownership, `register_rejects_a_manager_at_registration` for the field the commitment never covered, and the property `property_a_commitment_must_be_matured_relative_and_the_registrants_own` over both.
