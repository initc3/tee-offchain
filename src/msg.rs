use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    IterateHash {
        counter: Uint128,
        current_hash: Binary,
        old_mac: Binary,
    },
    ApplyUpdate {
        new_counter: Uint128,
        new_hash: Binary,
    }
}

/// QueryMsg that the contract exposes
#[cw_serde]
pub enum QueryMsg {
    GetState {},
}

/// queries using permits instead of viewing keys
#[cw_serde]
pub enum QueryWithPermit {
    Permissioned {},
}

#[cw_serde]
pub enum QueryAnswer {
    QueryExAns {},
    ViewingKeyError { error: String },
}


#[cw_serde]
pub struct GetStateAnswer {
    pub counter: Uint128,
    pub current_hash: Binary,
    pub future_mac: Binary,
}
