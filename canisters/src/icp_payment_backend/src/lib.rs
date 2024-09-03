#[macro_use]
extern crate ic_cdk;

use std::cell::RefCell;

use candid::{candid_method, CandidType, Nat, Principal};
use icrc_ledger_types::{
    icrc1::{
        account::Account,
        transfer::{BlockIndex, Memo, NumTokens, TransferArg, TransferError},
    },
    icrc2::{
        allowance::{Allowance, AllowanceArgs},
        approve::{ApproveArgs, ApproveError},
        transfer_from::{TransferFromArgs, TransferFromError},
    },
};
use serde::{Deserialize, Serialize};

mod error;
mod icpswap;
mod log;
mod merchant;
mod owners;
mod swaps;
mod token;

use error::{Error, Result};
use icpswap::*;
use log::*;
use owners::*;
use swaps::*;
use token::*;

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

const DEFAULT_SWAP_TOKEN_CANISTER_ID: &str = "xevnm-gaaaa-aaaar-qafnq-cai";

#[derive(CandidType, Default, Serialize, Deserialize)]
struct State {
    owners: Owners,
    swaps: Swaps,
    logs: Logs,
}

#[init]
fn init() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.owners.add_owner(ic_cdk::caller());
    });
}

#[post_upgrade]
fn post_upgrade() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.owners.add_owner(ic_cdk::caller());
    });
}

fn is_owner() -> Result<()> {
    let is_owner = STATE.with(|state| {
        let state = state.borrow();
        state.owners.is_owner(ic_cdk::caller())
    });

    if !is_owner {
        return Err(Error::Forbidden);
    }

    Ok(())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct GetErrorLogsArgs {
    pub start: Option<usize>,
    pub length: Option<usize>,
}

#[query]
#[candid_method(query)]
fn get_logs(data: GetErrorLogsArgs) -> Vec<Log> {
    STATE.with(|state| {
        let state = state.borrow();
        state.logs.get_logs(data.start, data.length)
    })
}

#[query]
#[candid_method(query)]
fn get_owners() -> Vec<Principal> {
    STATE.with(|state| {
        let state = state.borrow();
        state.owners.get_owners()
    })
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct AddOwnerArgs {
    owner: Principal,
}

#[update]
#[candid_method(update)]
fn add_owner(data: AddOwnerArgs) -> Result<()> {
    is_owner()?;

    let is_owner_exists = STATE.with(|state| {
        let state = state.borrow();
        state.owners.is_owner(data.owner)
    });

    if is_owner_exists {
        return Err(Error::OwnerAlreadyExists);
    }

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.owners.add_owner(data.owner);
        state
            .logs
            .add_log(LogLevel::Info, format!("Owner added: {:?}", data.owner));
    });

    Ok(())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct RemoveOwnerArgs {
    owner: Principal,
}

#[update]
#[candid_method(update)]
fn remove_owner(data: RemoveOwnerArgs) -> Result<()> {
    is_owner()?;

    let is_owner_exists = STATE.with(|state| {
        let state = state.borrow();
        state.owners.is_owner(data.owner)
    });

    if !is_owner_exists {
        return Err(Error::OwnerNotFound);
    }

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.owners.remove_owner(data.owner);
        state
            .logs
            .add_log(LogLevel::Info, format!("Owner removed: {:?}", data.owner));
    });

    Ok(())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct AddSwapArgs {
    token0: Principal,
    token1: Principal,
    pool_canister: Principal,
}

#[update]
#[candid_method(update)]
async fn add_swap(data: AddSwapArgs) -> Result<()> {
    is_owner()?;

    let is_exists = STATE.with(|state| {
        let state = state.borrow();
        state.swaps.exists(&data.token0, &data.token1)
    });

    if is_exists {
        return Err(Error::SwapAlreadyExists);
    }

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state
            .swaps
            .add_swap(data.token0, data.token1, data.pool_canister);
        state.logs.add_log(
            LogLevel::Info,
            format!("Swap added: {:?} -> {:?}", data.token0, data.token1),
        );
    });

    Ok(())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct RemoveSwapArgs {
    token0: Principal,
    token1: Principal,
}

#[update]
#[candid_method(update)]
fn remove_swap(data: RemoveSwapArgs) -> Result<()> {
    is_owner()?;

    let is_swap_exists = STATE.with(|state| {
        let state = state.borrow();
        state.swaps.exists(&data.token0, &data.token1)
    });

    if !is_swap_exists {
        return Err(Error::SwapNotFound);
    }

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.swaps.remove_swap(&data.token0, &data.token1);
        state.logs.add_log(
            LogLevel::Info,
            format!("Swap removed: {:?} -> {:?}", data.token0, data.token1),
        );
    });

    Ok(())
}

#[query]
#[candid_method(query)]
fn get_swaps() -> Vec<(Principal, Principal)> {
    STATE.with(|state| {
        let state = state.borrow();
        state.swaps.get_swaps()
    })
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct PayArgs {
    pub amount: NumTokens,
    pub token: Principal,
    pub to_merchant: Principal,
    pub memo: u64,
}

#[update]
#[candid_method(update)]
async fn pay(data: PayArgs) -> Result<()> {
    let swap_token = Principal::from_text(DEFAULT_SWAP_TOKEN_CANISTER_ID).unwrap();

    let pool_canister_id = STATE.with(|state| {
        let state = state.borrow();
        state
            .swaps
            .get_pool_canister_id(&data.token, &swap_token)
            .copied()
    });

    let pool_canister_id = match pool_canister_id {
        Some(id) => id,
        None => return Err(Error::SwapTokenNotFound),
    };

    let transfer_from_args = TransferFromArgs {
        amount: data.amount.clone(),
        from: Account {
            owner: ic_cdk::caller(),
            subaccount: None,
        },
        to: Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },
        fee: None,
        memo: Some(Memo::from(data.memo)),
        spender_subaccount: None,
        created_at_time: None,
    };

    ic_cdk::call::<(TransferFromArgs,), (core::result::Result<BlockIndex, TransferFromError>,)>(
        data.token,
        "icrc2_transfer_from",
        (transfer_from_args.clone(),),
    )
    .await
    .map_err(|(_, message)| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!(
                    "Failed to call icrc2_transfer_from ({:?}): {:?}",
                    transfer_from_args, message
                ),
            );
        });

        Error::IcCdkError { message }
    })?
    .0
    .map_err(|e| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!(
                    "Failed to call icrc2_transfer_from ({:?}): {:?}",
                    transfer_from_args, e
                ),
            );
        });

        Error::IcCdkError {
            message: format!("{:?}", e),
        }
    })?;

    // Check allowance
    let allowance_args = AllowanceArgs {
        account: Account {
            owner: ic_cdk::id(),
            subaccount: None,
        },
        spender: Account {
            owner: pool_canister_id.clone(),
            subaccount: None,
        },
    };

    let allowance = ic_cdk::call::<(AllowanceArgs,), (Allowance,)>(
        data.token,
        "icrc2_allowance",
        (allowance_args.clone(),),
    )
    .await
    .map_err(|(_, message)| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!(
                    "Failed to call icrc2_allowance ({:?}): {:?}",
                    allowance_args, message
                ),
            );
        });

        Error::IcCdkError { message }
    })?
    .0;

    // Set allowance
    if allowance.allowance.lt(&data.amount) {
        let approve_args = ApproveArgs {
            spender: Account {
                owner: pool_canister_id,
                subaccount: None,
            },
            amount: data.amount.clone() * Nat::from(1_000 as u64),
            from_subaccount: None,
            memo: None,
            created_at_time: None,
            expected_allowance: None,
            expires_at: None,
            fee: None,
        };

        ic_cdk::call::<(ApproveArgs,), (core::result::Result<Nat, ApproveError>,)>(
            data.token,
            "icrc2_approve",
            (approve_args.clone(),),
        )
        .await
        .map_err(|(_, message)| {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!(
                        "Failed to call icrc2_approve ({:?}): {:?}",
                        approve_args, message
                    ),
                );
            });

            Error::IcCdkError { message }
        })?
        .0
        .map_err(|e| {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!("Failed to call icrc2_approve ({:?}): {:?}", approve_args, e),
                );
            });

            Error::IcCdkError {
                message: format!("{:?}", e),
            }
        })?;
    }

    // Deposit to pool
    let fee = get_fee(data.token).await.map_err(|(_, message)| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!("Failed to call get_fee ({:?}): {:?}", data.token, message),
            );
        });

        Error::IcCdkError { message }
    })?;

    let deposit_from_args = DepositFromArgs {
        token: data.token.to_string(),
        amount: data.amount.clone(),
        fee,
    };

    let deposit_from_result = deposit_from(pool_canister_id, deposit_from_args.clone())
        .await
        .map_err(|(_, message)| {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!(
                        "Failed to call icpswap deposit_from ({:?}): {:?}",
                        deposit_from_args, message
                    ),
                );
            });

            Error::IcCdkError { message }
        })?;

    let deposit_amount = match deposit_from_result {
        IcpResult::ok(deposit_from_result) => deposit_from_result,
        IcpResult::err(e) => {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!(
                        "Failed to call icpswap deposit_from ({:?}): {:?}",
                        deposit_from_args, e
                    ),
                );
            });

            return Err(Error::IcCdkError {
                message: format!("{:?}", e),
            });
        }
    };

    // Swap token
    let swap_args = SwapArgs {
        amount_in: deposit_amount.to_string(),
        amount_out_minimum: Nat::from(0 as u64).to_string(),
        zero_for_one: true,
    };

    let swap_result = swap(pool_canister_id, swap_args.clone())
        .await
        .map_err(|(_, message)| {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!(
                        "Failed to call icpswap swap ({:?}): {:?}",
                        swap_args, message
                    ),
                );
            });

            Error::IcCdkError { message }
        })?;

    let swapped_amount = match swap_result {
        IcpResult::ok(swap_result) => swap_result,
        IcpResult::err(e) => {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!("Failed to call icpswap swap ({:?}): {:?}", swap_args, e),
                );
            });

            return Err(Error::IcCdkError {
                message: format!("{:?}", e),
            });
        }
    };

    // Withdraw from pool
    let fee = get_fee(swap_token).await.map_err(|(_, message)| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!("Failed to call get_fee ({:?}): {:?}", swap_token, message),
            );
        });

        Error::IcCdkError { message }
    })?;

    let withdraw_args = WithdrawArgs {
        token: swap_token.to_string(),
        amount: swapped_amount.clone(),
        fee: fee.clone(),
    };

    let withdraw_result = withdraw(pool_canister_id, withdraw_args.clone())
        .await
        .map_err(|(_, message)| {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!(
                        "Failed to call icpswap withdraw ({:?}): {:?}",
                        withdraw_args, message
                    ),
                );
            });

            Error::IcCdkError { message }
        })?;

    let withdraw_amount = match withdraw_result {
        IcpResult::ok(withdraw_result) => withdraw_result,
        IcpResult::err(e) => {
            STATE.with(|state| {
                state.borrow_mut().logs.add_log(
                    LogLevel::Error,
                    format!(
                        "Failed to call icpswap withdraw ({:?}): {:?}",
                        withdraw_args, e
                    ),
                );
            });

            return Err(Error::IcCdkError {
                message: format!("{:?}", e),
            });
        }
    };

    // Transfer to merchant
    let transfer_args = TransferArg {
        amount: withdraw_amount.clone() + fee.clone(),
        fee: Some(fee),
        to: Account {
            owner: data.to_merchant.clone(),
            subaccount: None,
        },
        memo: Some(Memo::from(data.memo)),
        from_subaccount: None,
        created_at_time: None,
    };

    ic_cdk::call::<(TransferArg,), (core::result::Result<BlockIndex, TransferError>,)>(
        swap_token,
        "icrc1_transfer",
        (transfer_args.clone(),),
    )
    .await
    .map_err(|(_, message)| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!(
                    "Failed to call icrc1_transfer ({:?}): {:?}",
                    transfer_args, message
                ),
            );
        });

        Error::IcCdkError { message }
    })?
    .0
    .map_err(|e| {
        STATE.with(|state| {
            state.borrow_mut().logs.add_log(
                LogLevel::Error,
                format!(
                    "Failed to call icrc1_transfer ({:?}): {:?}",
                    transfer_args, e
                ),
            );
        });

        Error::IcCdkError {
            message: format!("{:?}", e),
        }
    })?;

    Ok(())
}

// #[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
// struct MerchantWithdrawArgs {
//     token: Principal,
//     amount: NumTokens,
//     to: Option<Principal>,
// }

// #[update]
// #[candid_method(update)]
// async fn merchant_withdraw(data: MerchantWithdrawArgs) -> Result<()> {
//     let caller = ic_cdk::caller();
//     let current_balance = STATE.with(|state| {
//         let state = state.borrow();
//         state.merchants.get_amount(caller, data.token.clone())
//     });

//     if current_balance.lt(&data.amount) {
//         return Err(Error::InsufficientBalance {
//             balance: current_balance,
//         });
//     }

//     let fee = get_fee(data.token)
//         .await
//         .map_err(|(_, message)| Error::IcCdkError { message })?;

//     let amount = data.amount.clone();

//     let transfer_args = TransferArg {
//         amount,
//         fee: Some(fee),
//         to: match data.to {
//             Some(to) => Account {
//                 owner: to,
//                 subaccount: None,
//             },
//             None => Account {
//                 owner: caller,
//                 subaccount: None,
//             },
//         },
//         memo: None,
//         from_subaccount: None,
//         created_at_time: None,
//     };

//     ic_cdk::call::<(TransferArg,), (core::result::Result<BlockIndex, TransferError>,)>(
//         data.token,
//         "icrc1_transfer",
//         (transfer_args,),
//     )
//     .await
//     .map_err(|(_, message)| Error::IcCdkError { message })?
//     .0
//     .map_err(|e| {
//         if let TransferError::InsufficientFunds { balance } = e {
//             return Error::InsufficientBalance { balance };
//         }

//         Error::IcCdkError {
//             message: format!("{:?}", e),
//         }
//     })?;

//     STATE.with(|state| {
//         let mut state = state.borrow_mut();
//         state
//             .merchants
//             .sub_amount(caller, data.token.clone(), data.amount.clone())
//     });

//     Ok(())
// }

// #[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
// struct GetMerchantBalanceArgs {
//     pub merchant: Principal,
//     pub token: Principal,
// }

// #[query]
// #[candid_method(query)]
// fn get_merchant_balance(data: GetMerchantBalanceArgs) -> NumTokens {
//     STATE.with(|state| {
//         let state = state.borrow();
//         state.merchants.get_amount(data.merchant, data.token)
//     })
// }

// #[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
// struct GetMerchantBalancesArgs {
//     pub merchant: Principal,
// }

// #[query]
// #[candid_method(query)]
// fn get_merchant_balances(data: GetMerchantBalancesArgs) -> Vec<(Principal, Nat)> {
//     STATE
//         .with(|state| {
//             let state = state.borrow();
//             state.merchants.get_merchant_amounts(data.merchant)
//         })
//         .into_iter()
//         .map(|(token, amount)| (token, amount))
//         .collect()
// }

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct WithdrawBalanceArgs {
    pub token: Principal,
    pub amount: NumTokens,
    pub to: Principal,
}

#[update]
#[candid_method(update)]
async fn withdraw_balance(data: WithdrawBalanceArgs) -> Result<()> {
    is_owner()?;

    let transfer_args = TransferArg {
        amount: data.amount.clone(),
        to: Account {
            owner: data.to.clone(),
            subaccount: None,
        },
        fee: None,
        memo: None,
        from_subaccount: None,
        created_at_time: None,
    };

    ic_cdk::call::<(TransferArg,), (core::result::Result<BlockIndex, TransferError>,)>(
        data.token,
        "icrc1_transfer",
        (transfer_args,),
    )
    .await
    .map_err(|(_, message)| Error::IcCdkError { message })?
    .0
    .map_err(|e| {
        if let TransferError::InsufficientFunds { balance } = e {
            return Error::InsufficientBalance { balance };
        }

        Error::IcCdkError {
            message: format!("{:?}", e),
        }
    })?;

    Ok(())
}

export_candid!();
