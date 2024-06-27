NETWORK="local"

dfx canister create ckETH_ledger --network "${NETWORK}"
dfx canister create ICP_ledger --network "${NETWORK}"
dfx canister create ckBTC_ledger --network "${NETWORK}"

dfx canister create ckBTC_Index --network "${NETWORK}"
dfx canister create ICP_Index --network "${NETWORK}"
dfx canister create ckETH_Index --network "${NETWORK}"




dfx identity use testID

MINTERID="$(dfx identity get-principal)"
echo $MINTERID

PRINCIPAL="$(dfx identity get-principal)"
echo $PRINCIPAL

export MINTER_ACCOUNT_ID=$(dfx ledger account-id)

export DEFAULT_ACCOUNT_ID=$(dfx ledger account-id)


CKBTCLEDGERID="$(dfx canister id ckBTC_ledger --network "${NETWORK}")"
echo $CKBTCLEDGERID

CKETHLEDGERID="$(dfx canister id ckETH_ledger --network "${NETWORK}")"
echo $CKBTCLEDGERID

ICPLEDGERID="$(dfx canister id ICP_ledger --network "${NETWORK}")"
echo $ICPLEDGERID


## Deploy the ckBTC index canister
echo "Deploying the ckBTC Index canister"
dfx deploy --network "${NETWORK}" ckBTC_Index --argument '
  record {
   ledger_id = (principal "'${CKBTCLEDGERID}'");
  }
' --mode=reinstall -y

## Deploy the ckETH index canister
echo "Deploying the ckETH Index canister"
dfx deploy --network "${NETWORK}" ckETH_Index --argument '
  record {
   ledger_id = (principal "'${CKETHLEDGERID}'");
  }
' --mode=reinstall -y


## Deploy the ckBTC index canister
echo "Deploying the ICP Index canister"
dfx deploy --network "${NETWORK}" ICP_Index --argument '
  record {
   ledger_id = (principal "'${ICPLEDGERID}'");
  }
' --mode=reinstall -y

























## deploy the chat ledger canister
echo "Step 2: deploying ICP_ledger canister..."
dfx deploy  ICP_ledger --argument "
  (variant {
    Init = record {
      minting_account = \"$MINTER_ACCOUNT_ID\";
      initial_values = vec {
        record {
          \"$DEFAULT_ACCOUNT_ID\";
          record {
            e8s = 10_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10_000 : nat64;
      };
      token_symbol = opt \"ICP\";
      token_name = opt \"Local ICP\";
    }
  })
" --mode=reinstall -y


## deploy the ckbtc ledger canister
echo "Step 6: deploying ckBTC_ledger canister......."
dfx deploy --network "${NETWORK}" ckBTC_ledger --argument '
  (variant {
    Init = record {
      token_name = "Testnet ckBTC";
      token_symbol = "ckBTC";
      minting_account = record { owner = principal "'${MINTERID}'";};
      initial_balances = vec { record { record { owner = principal "'${MINTERID}'";}; 100_000_000_000; }; };
      metadata = vec {};
      transfer_fee = 10;
      archive_options = record {
        trigger_threshold = 2000;
        num_blocks_to_archive = 1000;
        controller_id = principal "'${PRINCIPAL}'";
      }
    }
  })
' --mode=reinstall -y


## deploy the ckbtc ledger canister
echo "Step 6: deploying ckETH_ledger canister......."
dfx deploy --network "${NETWORK}" ckETH_ledger --argument '
  (variant {
    Init = record {
      token_name = "Testnet ckETH";
      token_symbol = "ckETH";
      minting_account = record { owner = principal "'${MINTERID}'";};
      initial_balances = vec { record { record { owner = principal "'${MINTERID}'";}; 100_000_000_000; }; };
      metadata = vec {};
      transfer_fee = 10;
      archive_options = record {
        trigger_threshold = 2000;
        num_blocks_to_archive = 1000;
        controller_id = principal "'${PRINCIPAL}'";
      }
    }
  })
' --mode=reinstall -y