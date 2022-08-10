## K-Pay solution

Read all transactions from a source file(default is: transaction.csv) and build client accounts in specified target file (account.csv).

## Assumptions
- Account is created with the 1st client's `deposit` transaction. Beforehad any other client's transaction is ignored.
- The dispute is either `resolved` or `chargedback`. The flow `dispute`->`resolve` -> `chargeback` is not implemented in this solution.
- Locking the account (`frozen`) after chargeback does not have any further effect on next/future transactions. Deposits and withdraws are allowed.
- Client and transactions (client_id, tx_id) are unique. If duplicate is found it is ignored.
- There could only one dispute for a transaction. If duplicated is found it is ignored.
- Chargeback is implement following the requirement: 
    `This means that the clients held funds and total funds should decrease by the amount previously disputed.`
### Note
Misleading chargeback requirement.
 1. charrgeback represents the cleint `REVERSING` transaction.
 2. `This means that the clients held funds and total funds should decrease by the amount previously disputed.`
If chargeback is a reversing action then no more details are needed as it is clear that we should apply the opposite action:
 deposit -> withdraw
 withdraw -> deposit

In this case both conditions are self excluding.

In case of `withdrawl` requirement #2. is wrong as it forces next withdrawl with the same amount.
In case of `deposit` that works as client expects.

## Run the app
cargo build
`cargo run -- transactions.csv > accounts.csv`

## Examples

## Enjoy :smiley:
