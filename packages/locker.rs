use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use cw20_base::msg::QueryMsg as QueryCw20Msg;

#[cw_serde]
pub enum TokenInfo {
    CW20Token { contract_addr: Addr },
    NativeToken { denom: String },
}

impl TokenInfo {
    pub fn get_as_bytes(&self) -> &[u8] {
        match self {
            TokenInfo::CW20Token { contract_addr } => contract_addr.as_bytes(),
            TokenInfo::NativeToken { denom } => denom.as_bytes(),
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub fees_address: Addr,
    pub lock_fees: u128,
}

#[cw_serde]
pub struct Token {
    pub info: TokenInfo,
    pub amount: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Lock { amount: Uint128, token: TokenInfo, token_type: String, duration: u64 },
    Unlock { index: u128 },
    SetLockFee { amount: u128 }
}

#[cw_serde]
pub enum ExecutePairMsg {
    TokenExecute(Cw20ExecuteMsg)
}

#[cw_serde]
pub enum QueryPairMsg {
    TokenQuery(QueryCw20Msg)
}

#[cw_serde]
pub struct Config {
    pub fees_address: Addr,
    pub lock_fees: u128,
    pub owner: Addr,
}

#[cw_serde]
pub struct UserEntry {
    pub amount_locked: Uint128,
    pub token: TokenInfo,
    pub locked_at: u64,
    pub unlock_at: u64,
    pub token_type: String
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(UserEntry)]
    LockedTokensByIndex { account: Addr, index: u128 },
    #[returns(u128)]
    NumberOfLocks { account: Addr },
}
