# The case study

Five entries that only make sense against one particular script's structure: the Cells
contracts, the ring of `.cell` names on Nervos CKB, public since 2026-09-23 at
[cells-contracts](https://github.com/LusoCryptoLabs/cells-contracts). They were written
for this catalogue in September 2026 and held back until the code they describe could be
read. Each is a rewrite of a finding in that repository's security journal, in the same
pattern, transaction, fix, test shape as the general entries, and each ends with the tests
that pin it.

| # | Entry | The one-line version |
|---|---|---|
| [04](04-a-new-cells-lock-is-not-pinned.md) | A new cell's lock is not pinned | Anyone registers a cell under an unspendable lock and its whole id range is frozen for ever |
| [05](05-the-sentinel-is-a-cell-too.md) | The sentinel is a cell too | "Id is zero" is not "the root": recycle it and any label can be minted at id zero |
| [06](06-a-commitment-is-public-the-moment-it-lands.md) | A commitment is public the moment it lands | Copy the thirty-two bytes, mature them on the same clock, and front-run the reveal |
| [08](08-a-deposit-paid-back-to-the-wrong-side.md) | A deposit paid back to the wrong side | A buyer paid the seller with the seller's own listing deposit; found as a real sale |
| [14](14-a-child-cell-buys-its-own-future.md) | A child cell buys its own future | Permissionless renewal let a sub-name outlive its parent's change of hands |

The numbers are the catalogue's: the general entries and these were found in one sequence
and are numbered in it.
