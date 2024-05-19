use crate::state::{CONFIG, USER_INFO, TOKENS_LOCKED};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_json_binary, DepsMut, Env, MessageInfo, Response, StdError, StdResult
};
use cw20::Cw20ExecuteMsg;
use packages::locker::{UserEntry, ExecutePairMsg};

use crate::query_pt::query;

pub mod execute {

    use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Empty, Uint128, WasmMsg};

    use super::*;
    use packages::locker::TokenInfo;

    pub fn set_lock_fee(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        amount: u128
    ) -> StdResult<Response> {
    
        let mut config = CONFIG.load(deps.storage)?;
    
        if _info.sender != config.owner {
            return Err(StdError::generic_err("Unauthorized caller"));
        }
        
        config.lock_fees = amount;
    
        CONFIG.save(deps.storage, &config)?;
    
        Ok(Response::default())
    }

    pub fn lock(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
        token: TokenInfo,
        token_type: String,
        duration: u64
    ) -> StdResult<Response> {

        let account = info.sender.clone();

        let mut res = Response::new();

        let tokens_locked;

        let config = CONFIG.load(deps.storage).unwrap();

        if (TOKENS_LOCKED.may_load(deps.storage, &account)?).is_some() {
            tokens_locked = TOKENS_LOCKED.load(deps.storage, &account).unwrap();
        }
        else {
            tokens_locked = 0;
        }

        let has_paid_fees = info.funds.iter().any(|coin| {
            coin.denom == String::from("unibi") && coin.amount.u128() >= config.lock_fees
        });

        if !has_paid_fees {
            return Err(StdError::generic_err("Insufficient fees paid"));
        }

        if token_type == String::from("nitoshidex_lp") {
            match &token {
                TokenInfo::CW20Token { contract_addr } => {
                    let asset_transfer = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        msg: to_json_binary(&ExecutePairMsg::TokenExecute(
                            Cw20ExecuteMsg::TransferFrom {
                                owner: account.to_string(),
                                recipient: env.contract.address.to_string().clone(),
                                amount: amount.into()
                            },
                        ))?,
                        funds: vec![],
                    });

                    let fees = deduct_fees(
                        info.clone(),
                        config.fees_address,
                        config.lock_fees
                    );

                    res = res.add_message(asset_transfer).add_message(fees);
                    
                }
                TokenInfo::NativeToken { denom: _denom } => {
                    return Err(StdError::generic_err("Invalid token lock"));
                }
            };
        }
        else {
            match &token {
                TokenInfo::CW20Token { contract_addr } => {
    
                    let asset_transfer = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                            owner: account.clone().to_string(),
                            recipient: env.contract.address.to_string().clone(),
                            amount: amount.into(),
                        })?,
                        funds: vec![],
                    });
    
                    let fees = deduct_fees(
                        info.clone(),
                        config.fees_address,
                        config.lock_fees
                    );

                    res = res.add_message(asset_transfer).add_message(fees);

                }
                TokenInfo::NativeToken { denom: _denom } => {

                    let required_amount = amount + Uint128::from(config.lock_fees);
    
                    let sent_sufficient_funds = info.funds.iter().any(|coin| {
                        return *_denom == String::from("unibi") && coin.amount == required_amount;
                    });
    
                    if !sent_sufficient_funds {
                        return Err(StdError::generic_err("Insufficient funds sent"));
                    }
                    
                    let fees = deduct_fees(
                        info.clone(),
                        config.fees_address,
                        config.lock_fees
                    );

                    res = res.add_message(fees);
                    
                }
            };
        }

        let locked_at = env.block.time.seconds();

        let unlock_at = locked_at + duration;

        let user_info = UserEntry {
            amount_locked: amount,
            token,
            locked_at,
            unlock_at,
            token_type
        };

        TOKENS_LOCKED.save(deps.storage, &account, &(tokens_locked + 1))?;

        USER_INFO.save(deps.storage, &(account, tokens_locked), &user_info)?;

        Ok(res)
    }

    pub fn unlock(deps: DepsMut, env: Env, info: MessageInfo, index: u128) -> StdResult<Response> {
        
        let account = info.sender.clone();

        let mut res = Response::new();

        let user_info = USER_INFO.load(deps.storage, &(account, index)).unwrap();

        if user_info.unlock_at > env.block.time.seconds() {
            return Err(StdError::generic_err("Unlock date still in the future"));
        }

        let tokens_locked = TOKENS_LOCKED.load(deps.storage, &info.sender.clone()).unwrap();

        let this_address = env.contract.address.clone();

        let last_lock = USER_INFO.load(deps.storage, &(info.sender.clone(), tokens_locked - 1)).unwrap();

        if user_info.token_type == String::from("nitoshidex_lp") {
            match &user_info.token {
                TokenInfo::CW20Token { contract_addr } => {

                    let balance = query::query_token_balance(
                        &deps.querier,
                        contract_addr.clone(),
                        this_address.clone(),
                        String::from("nitoshidex_lp")
                    )
                    .unwrap();

                    let mut send_amount = user_info.amount_locked;
                    if send_amount >= balance {
                        send_amount = balance;
                    }

                    let asset_transfer = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        msg: to_json_binary(&ExecutePairMsg::TokenExecute(
                            Cw20ExecuteMsg::Transfer {
                                recipient: info.sender.clone().into(),
                                amount: send_amount.into()
                            },
                        ))?,
                        funds: vec![],
                    });

                    res = res.add_message(asset_transfer);
                    
                }
                TokenInfo::NativeToken { denom: _denom } => {
                    return Err(StdError::generic_err("Invalid token lock"));
                }
            };
        }

        else {
            match &user_info.token {
                TokenInfo::CW20Token { contract_addr } => {

                    let balance = query::query_token_balance(
                        &deps.querier,
                        contract_addr.clone(),
                        this_address.clone(),
                        String::from("token")
                    )
                    .unwrap();

                    let mut send_amount = user_info.amount_locked;
                    if send_amount >= balance {
                        send_amount = balance;
                    }
    
                    let asset_transfer = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: info.sender.clone().into(),
                            amount: send_amount.into()
                        })?,
                        funds: vec![],
                    });

                    res = res.add_message(asset_transfer);

                }
                TokenInfo::NativeToken { denom } => {
                    let native_asset_transfer: CosmosMsg<Empty> = CosmosMsg::Bank(BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: vec![Coin {
                            denom: denom.to_string().clone(),
                            amount: user_info.amount_locked
                        }],
                    });
                    res = res.add_message(native_asset_transfer);
                }
            };
        }

        TOKENS_LOCKED.save(deps.storage, &info.sender.clone(), &(tokens_locked - 1))?;

        USER_INFO.save(deps.storage, &(info.sender.clone(), index), &last_lock)?;

        USER_INFO.remove(deps.storage, &(info.sender.clone(), tokens_locked - 1));

        Ok(res)
    }

    fn deduct_fees(
        _info: MessageInfo,
        receiver: Addr,
        amount: u128,
    ) -> CosmosMsg {

        let message: CosmosMsg<Empty> = CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string().clone(),
            amount: vec![Coin {
                denom: String::from("unibi").clone(),
                amount: amount.into(),
            }],
        });

        message
    }
}
