use crate::state::{CONFIG, USER_INFO, TOKENS_LOCKED};
use cosmwasm_std::{to_json_binary, Deps, Env, StdResult};
use cw20::BalanceResponse as CW20_BalanceResponse;
use packages::locker::{Config, UserEntry, QueryPairMsg};

pub mod query {
    use super::*;
    use cosmwasm_std::{
        Addr, QuerierWrapper, QueryRequest, Uint128, WasmQuery,
    };

    use cw20::Cw20QueryMsg;
    use cw20_base::msg::QueryMsg as QueryCw20Msg;

    pub fn query_pair_info(deps: Deps) -> StdResult<Config> {
        let config: Config = CONFIG.load(deps.storage).unwrap();
        Ok(config)
    }

    pub fn query_user_info(
        deps: Deps,
        _env: Env,
        account: Addr,
        index: u128,
    ) -> StdResult<UserEntry> {
        let user_info: UserEntry = USER_INFO.load(deps.storage, &(account, index)).unwrap();
        Ok(user_info)
    }

    pub fn query_locked_tokens(
        deps: Deps,
        _env: Env,
        account: Addr
    ) -> StdResult<u128> {
        let tokens_locked;
        if (TOKENS_LOCKED.may_load(deps.storage, &account)?).is_some() {
            tokens_locked = TOKENS_LOCKED.load(deps.storage, &account).unwrap();
        }
        else {
            tokens_locked = 0;
        }
        Ok(tokens_locked)
    }

    pub fn query_token_balance(
        querier: &QuerierWrapper,
        contract_addr: Addr,
        account_addr: Addr,
        token_type: String
    ) -> StdResult<Uint128> {
    
        let res: CW20_BalanceResponse;
        
        if token_type == String::from("token") {
            res = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&Cw20QueryMsg::Balance {
                    address: account_addr.to_string(),
                })?,
            }))?;
        }
        else {
            res = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&QueryPairMsg::TokenQuery(
                    QueryCw20Msg::Balance {
                        address: account_addr.to_string(),
                    },
                ))?
            }))?;
        }
        // load balance form the token contract
        Ok(res.balance)
    }
}
