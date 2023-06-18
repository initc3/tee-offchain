use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Binary, Uint128};
use secret_toolkit::storage::Item;

/// Basic configuration struct
pub static CONFIG_KEY: Item<State> = Item::new(b"config");

// Requests
pub static PREFIX_REQUESTS_KEY: Item<ReqState> = Item::new(b"requests");
pub static PREFIX_RESPONSE_KEY: Item<RespState> = Item::new(b"responses");
pub static PREFIX_CHECKPOINT_KEY: Item<CheckPtState> = Item::new(b"checkpoints");
pub static REQUEST_SEQNO_KEY: Item<Uint128> = Item::new(b"request_seqno");
pub static CHECKPOINT_SEQNO_KEY: Item<Uint128> = Item::new(b"check_seqno");

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub key: Binary,
    pub current_hash: Binary,
    pub counter: Uint128,
}

#[cw_serde]
pub struct ReqState {
    pub reqtype: ReqType,
    pub from: Addr,
    pub to: Option<Addr>,
    pub amount: Uint128,
    pub memo: Option<String>
}

#[cw_serde]
pub enum ReqType {
    DEPOSIT,
    TRANSFER,
    WITHDRAW
}

#[cw_serde]
pub struct RespState {
    pub seqno: Uint128,
    pub status: bool,
    pub amount: Uint128,
    pub response: String
}

#[cw_serde]
pub struct CheckPtState {
    pub balance: Uint128,
    pub address: Addr
}
