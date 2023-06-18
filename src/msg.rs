use cosmwasm_schema::{cw_serde, QueryResponses};
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Grab the state for a querier
    #[returns(GetStateResp)]
    GetState {},
    /// Iterate upwards bc cool
    #[returns(IterateHashResp)]
    IterateHash {
        counter: Uint128,
        current_hash: Binary,
        old_mac: Binary,
    },
}

#[cw_serde]
pub struct GetStateResp {
    pub counter: Uint128,
    pub current_hash: Binary,
    pub current_mac: Binary,
}

#[cw_serde]
pub struct IterateHashResp {
    pub new_counter: Uint128,
    pub new_hash: Binary,
    pub new_mac: Binary
}