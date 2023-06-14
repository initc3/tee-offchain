use std::ops::DerefMut;
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, Storage, Uint128, StdResult};
// use secret_toolkit::utils::pad_handle_result;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use secret_toolkit::crypto::sha_256;
use serde::Serialize;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg};
use crate::state::{State, CONFIG_KEY};

type HmacSha256 = Hmac<Sha256>;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    // grab random entropy that is produced by the consensus
    let entropy = env.block.random.unwrap();

    // The `State` is created
    let config = State {
        owner: info.sender,
        current_hash: entropy,
        counter: Uint128::zero()
    };

    // Save data to storage
    CONFIG_KEY.save(deps.storage, &config).unwrap();

    // CONFIG_KEY.save(deps.storage.deref_mut(), &config).unwrap();

    Ok(Response::new())
}

//-------------------------------------------- HANDLES ---------------------------------

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let res = match msg {
        ExecuteMsg::ApplyUpdate {
            new_counter,
            new_hash
        } => {


            Ok(Response::new())
        },
        ExecuteMsg::IterateHash {
            counter,
            current_hash,
            old_mac
        } => {
            Ok(Response::new())
        }
    };
    res
}

fn try_apply_update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_counter: Uint128,
    new_hash: Binary,
) -> StdResult<Response> {



    Ok(Response::new())
}

fn try_iterate_hash(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    counter: Uint128,
    current_hash: Binary,
    old_mac: Binary
) -> StdResult<Response> {


    Ok(Response::new())
}

/// Returns Result<Response, ContractError>
///
/// create a viewing key
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
/// * `entropy` - string to be used as an entropy source for randomization
fn try_create_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    // let key = ViewingKey::create(
    //     deps.storage,
    //     &info,
    //     &env,
    //     info.sender.as_str(),
    //     entropy.as_bytes(),
    // );

    Ok(Response::new())
}

// ---------------------------------------- QUERIES --------------------------------------

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = match msg {
        QueryMsg::GetState {} => {

        }
    };
    Ok(Binary::default())
}

fn query_ex(_deps: Deps) -> StdResult<Binary> {
    Ok(to_binary(&QueryAnswer::QueryExAns {})?)
}

fn query_permissioned(_deps: Deps, _viewer: String) -> StdResult<Binary> {
    Ok(to_binary(&QueryAnswer::QueryExAns {})?)
}

fn gen_hash(deps: Deps, counter_in: Uint128) -> StdResult<Binary> {
    // Counter
    let counter_as_bytes = counter_in.to_le_bytes();

    // Hashed result
    let hash = sha_256(counter_as_bytes.as_slice());
}

/// Returns a view of the state. Returns the current counter, new hash, mac
fn gen_mac(deps: Deps, env: Env) -> StdResult<(Uint128, Binary, Binary)> {
    // in theory we've already instantiated the contract so this cannot fail...
    let store = CONFIG_KEY.load(deps.storage).unwrap();

    // return the the counter,
    Ok((Uint128::zero(), Binary::default(), Binary::default()))
}