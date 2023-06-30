use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128};


#[cw_serde]
pub struct InstantiateMsg {
    pub entropy: Binary // Since user doesn't trust the backend, they spice it up by themselves provide fat entropy to bootstrap the contract's source of randomness but as TEE'd state
}

#[cw_serde]
pub enum ExecuteMsg { // This entire ExecuteMessage should be padded to a potentially large number of a certain base then an even larger number
    ApplyUpdate { // Apply the state machine's cipertext into the TEE's internal encrypted state. This is done by writing it back, this means the user is willing to share the inner value. The user commits to the internal value effectively sharing it to the safe in the safe.
        new_counter: Uint128, // The value to update in state
        new_hash: Binary, // The hash that we're going to commit to onto chain
        current_mac: Binary // The previous message authentication code, this is literally just hash of the previous state value.
    }
}

/// QueryMessage
#[cw_serde]
pub enum QueryMsg {
    /// Lets politely ask the safe in a safe for its contents.
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