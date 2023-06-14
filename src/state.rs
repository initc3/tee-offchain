use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Binary, Uint128};
use secret_toolkit::storage::Item;

/// Basic configuration struct
pub static CONFIG_KEY: Item<State> = Item::new(b"config");
/// Revoked permits prefix key
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";
/// pad handle responses and log attributes to blocks of 256 bytes to prevent leaking info based on
/// response size
pub const BLOCK_SIZE: usize = 256;

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub current_hash: Binary,
    pub counter: Uint128,
}
