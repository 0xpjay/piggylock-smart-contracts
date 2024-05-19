use crate::state::CONFIG;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult
};
use packages::locker::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::execute_pt::execute;
use crate::query_pt::query;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config: Config = Config {
        fees_address: msg.fees_address,
        lock_fees: msg.lock_fees,
        owner: _info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Lock { amount, token, token_type, duration } => execute::lock(deps, env, info, amount, token, token_type, duration),
        ExecuteMsg::Unlock { index } => execute::unlock(deps, env, info, index),
        ExecuteMsg::SetLockFee { amount } => execute::set_lock_fee(deps, env, info, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query::query_pair_info(deps)?),
        QueryMsg::NumberOfLocks { account } => {
            to_json_binary(&query::query_locked_tokens(deps, env, account)?)
        }
        QueryMsg::LockedTokensByIndex { account, index } => {
            to_json_binary(&query::query_user_info(deps, env, account, index)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        0u64 => reply::instantiate_reply(deps, env, msg),
        _ => Ok(Response::default()),
    }
}

pub mod reply {
    use super::*;

    pub fn instantiate_reply(_deps: DepsMut, _env: Env, _msg: Reply) -> StdResult<Response> {
        Ok(Response::new())
    }
}
