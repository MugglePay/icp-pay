

for the split payments

- monitor the balance of the backend canister periodically. if the balance increases, fetch the latest transaction that caused the balance to increase
the transaction contains the sender and the amount.

return 99% of the received funds back to the sender.
get all the addresses that need to shared the 1% and send the respective percentages to these addresses.

-- for the recurring payments, let the user depsit money into the account controlled by the backend canister so that whenever the month ends, the canister can automatically deduct the amount.

for tests ??????


ICP ledger canister local - bd3sg-teaaa-aaaaa-qaaba-cai      ryjl3-tyaaa-aaaaa-aaaba-cai
ckETH ledger canister local - bkyz2-fmaaa-aaaaa-qaaaq-cai   ss2fx-dyaaa-aaaar-qacoq-cai
ckBTC ledger canister local - be2us-64aaa-aaaaa-qaabq-cai   mxzaz-hqaaa-aaaar-qaada-cai



dfx canister --network local call ckETH_ledger icrc1_transfer '
  (record {
    to=(record {
      owner=(principal "bw4dl-smaaa-aaaaa-qaacq-cai")
    });
    amount=10_000_000
  })
'