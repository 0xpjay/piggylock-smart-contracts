use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use packages::locker::{Config, UserEntry};

pub const CONFIG: Item<Config> = Item::new("config");
pub const USER_INFO: Map<&(Addr, u128), UserEntry> = Map::new("user info");
pub const TOKENS_LOCKED: Map<&Addr, u128> = Map::new("tokens_locked");
