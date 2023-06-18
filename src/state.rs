use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Binary, Uint128, Env, StdResult, Storage, StdError, to_binary, from_binary};
use secret_toolkit::storage::Item;
use crate::utils::{encrypt_with_nonce, decrypt, get_nonce};

/// Basic configuration struct
pub static CONFIG_KEY: Item<State> = Item::new(b"config");

// Requests
pub static PREFIX_REQUESTS_KEY: Item<Request> = Item::new(b"requests");
pub static PREFIX_RESPONSE_KEY: Item<ResponseState> = Item::new(b"responses");
pub static REQUEST_SEQNO_KEY: Item<Uint128> = Item::new(b"request_seqno");
pub static CHECKPOINT_SEQNO_KEY: Item<Uint128> = Item::new(b"check_seqno");
pub static CHECKPOINT_KEY: Item<CheckPoint> = Item::new(b"checkpoint");
pub static AEAD_KEY: Item<SymmetricKey> = Item::new(b"aead_key");

pub type SymmetricKey = [u8; 32];

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub key: Binary,
    pub current_hash: Binary,
    pub counter: Uint128,
}

#[cw_serde]
pub struct Request {
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
pub struct ResponseState {
    pub seqno: Uint128,
    pub status: bool,
    pub amount: Uint128,
    pub response: String
}

#[cw_serde]
pub struct AddressBalance {
    pub balance: Uint128,
    pub address: Addr
}

impl Request {
    pub fn load(store: &dyn Storage, seqno: Uint128) -> StdResult<Request> {
        let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());
        req_key.load(store).map_err(|_err| StdError::generic_err("Request load error"))
    }

    pub fn save(store: &mut dyn Storage, request: Request, seqno: Uint128) -> StdResult<()> {
        let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());
        req_key.save(store, &request)
    }
}
impl ResponseState {
    pub fn decrypt_response(store: &dyn Storage, cipher: Binary) -> StdResult<ResponseState> {
        let key = AEAD_KEY.load(store).unwrap();
        let resp_vec = decrypt(cipher.as_slice(), &key).unwrap();
        let resp_bin = to_binary(&resp_vec).unwrap();
        from_binary(&resp_bin)
    }

    pub fn encrypt_response(store: &dyn Storage, env: Env, response: ResponseState) -> StdResult<Binary> {
        let key = AEAD_KEY.load(store).unwrap();
        let plaintext = to_binary(&response).unwrap();
        let nonce = get_nonce(env);
        let cipher = encrypt_with_nonce(&plaintext, &key, &nonce).unwrap();
        let response_bin = to_binary(&cipher);
        return response_bin;
    }
}

#[cw_serde]
pub struct CheckPoint {
    pub checkpoint: Vec<AddressBalance>,
    pub seqno: Uint128
}

impl CheckPoint {
    pub fn load(store: &dyn Storage) -> StdResult<CheckPoint> {
        CHECKPOINT_KEY.load(store).map_err(|_err| StdError::generic_err("Checkpoint load error"))
    }

    pub fn save(store: &mut dyn Storage, checkpoint: CheckPoint) -> StdResult<()> {
        CHECKPOINT_KEY.save(store, &checkpoint)
    }

    pub fn decrypt_checkpoint(store: &dyn Storage, cipher: Binary) -> StdResult<CheckPoint> {
        let key = AEAD_KEY.load(store).unwrap();
        let checkpoint_vec = decrypt(cipher.as_slice(), &key).unwrap();
        let checkpoint_bin = to_binary(&checkpoint_vec).unwrap();
        from_binary(&checkpoint_bin)
    }

    pub fn encrypt_checkpoint(store: &dyn Storage, env: Env, checkpoint: CheckPoint) -> StdResult<Binary> {
        let key = AEAD_KEY.load(store).unwrap();
        let plaintext = to_binary(&checkpoint).unwrap();
        let nonce = get_nonce(env);
        let cipher = encrypt_with_nonce(&plaintext, &key, &nonce).unwrap();
        let checkpoint_bin = to_binary(&cipher);
        return checkpoint_bin;
    }
}