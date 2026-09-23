# 08. A deposit paid back to the wrong side

**Class:** an input's capacity flowing to whoever builds the transaction. **Severity:**
High where the deposit exceeds the price; found as a real sale on a public network.

## The pattern

A listing parks a deposit in an offer cell so the listing is discoverable on chain. The
sale lock on that cell unlocks when the outputs pay the seller at least the price. The
offer cell's own capacity was deliberately **not** counted as a payment, and the comment
said whoever completes the sale keeps it, "which is roughly what the transaction costs
them".

The transaction costs 0.00002 CKB. The deposit was 160. Nothing required the buyer to
bring anything: the check was on the outputs to the seller, and the offer cell's capacity
was already in the inputs. So for any price below the deposit, a stranger spent the offer
cell, paid the seller the asking price out of the seller's own money, and kept the rest
along with the name.

It was unreachable while the minimum listing was four times the deposit. Lowering the
minimum to make cheap names sellable opened it the same day, and it was found by taking a
transaction that had just been called a success and asking **where the buyer's money came
from**: their wallet had contributed zero.

## The fix

The offer cell's capacity is added to what the seller is owed, so it comes back to them in
the same transaction that sells the name. The buyer then brings exactly the price and the
seller receives exactly the price, which is what both were told. Eight existing tests
failed on the change and were right to: every one of them encoded the old split.

## The question to ask

For every input the script unlocks: **whose money is it, and where does the validator let
it go?** And for every "roughly" in a comment: measure it once. Here the estimate was
wrong by a factor of eight million.

## The test

A purchase in which the buyer's inputs contribute nothing: refused. Control: the honest
sale pays the seller price plus deposit. Then the property (see [13](../general/13-examples-are-not-properties.md)): the lock accepts a
purchase exactly when the seller receives at least `price - fee + deposit` and the
treasury at least `fee`. With the fix reverted, the property fails on 66 of 225 points and
names the first in words.

## Where it lives

The contracts are public at [cells-contracts](https://github.com/LusoCryptoLabs/cells-contracts). The tests that pin it: in [`tests/tests/sale_lock.rs`](https://github.com/LusoCryptoLabs/cells-contracts/blob/main/tests/tests/sale_lock.rs), `a_buyer_cannot_pay_the_seller_with_the_sellers_own_deposit` and its control `the_deposit_comes_back_to_the_seller_on_a_sale`; and in [`tests/tests/sale_properties.rs`](https://github.com/LusoCryptoLabs/cells-contracts/blob/main/tests/tests/sale_properties.rs), the grid, `the_lock_accepts_a_purchase_exactly_when_both_legs_are_paid` and `no_purchase_the_lock_accepts_leaves_the_seller_worse_off`.
