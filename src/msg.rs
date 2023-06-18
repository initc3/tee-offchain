use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ApplyUpdate {
        new_counter: Uint128,
        new_hash: Binary,
        current_mac: Binary
    }
}

/// QueryMsg that the contract exposes
#[cw_serde]
pub enum QueryMsg {
    /// Grab the state for a querier
    GetState {},
    /// Iterate upwards bc cool
    IterateHash {
        counter: Uint128,
        current_hash: Binary,
        old_mac: Binary,
    },
}

#[cw_serde]
pub struct GetStateAnswer {
    pub counter: Uint128,
    pub current_hash: Binary,
    pub current_mac: Binary,
}

#[cw_serde]
pub struct IterateHashAnswer {
    pub new_counter: Uint128,
    pub new_hash: Binary,
    pub new_mac: Binary
}