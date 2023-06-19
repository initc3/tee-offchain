use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128, Addr};
use crate::state::ReqType;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ApplyUpdate {
        new_counter: Uint128,
        new_hash: Binary,
        current_mac: Binary
    },

    SubmitDeposit {        
    },

    SubmitTransfer {
        to: Addr,
        amount: Uint128,
        memo: String,
    },

    SubmitWithdraw {
        amount: Uint128
    },

    CommitResponse {
        cipher: Binary
    },

    WriteCheckpoint {
        cipher: Binary       
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

    GetBalance{
        address: Addr
    },

    GetRequest {
        seqno: Uint128
    },

    GetCheckpoint {
    },
    ProcessNext {
        cipher: Binary
    }
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

#[cw_serde]
pub struct GetRequestAnswer {
    pub reqtype: ReqType,
    pub from: Addr
}

#[cw_serde]
pub struct ProcessResponseAnswer {
    pub request_cipher: Binary,
    pub checkpoint_ciper: Binary
}
