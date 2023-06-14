use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, Storage, Uint128, StdResult};
// use secret_toolkit::utils::pad_handle_result;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
// use secret_toolkit::crypto::sha_256;

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
        key: entropy.clone(),
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

// ---------------------------------------- QUERIES --------------------------------------

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let res = match msg {
        QueryMsg::GetState {} => {
            let store = CONFIG_KEY.load(deps.storage).unwrap();
            let future_mac = gen_mac(deps, env, state.counter, state.current_hash);
            GetStateAnswer {
                counter: state.counter,
                current_hash: state.current_hash,
                future_mac: gen_mac(deps, env, state.counter, state.current_hash)
            }
        }
    };
    Ok(Binary::default())
}

fn gen_hash(deps: Deps, counter_in: Uint128, current_hash: Binary) -> StdResult<Binary> {
    // Counter
    let store = CONFIG_KEY.load(deps.storage).unwrap();

    let mut hasher = Sha256::default();
    let counter_as_bytes = counter_in.to_le_bytes();

    hasher.update(counter_as_bytes.as_slice());
    hasher.update(current_hash.as_slice());

    let finalized_hash = hasher.finalize();
    let hash_digest = finalized_hash.as_slice();
    Ok(Binary::from(hash_digest))
}

/// Returns a view of the state. Returns the current counter, new hash, mac
fn gen_mac(deps: Deps, env: Env, counter_in: Uint128, current_hash: Binary) -> StdResult<(Uint128, Binary, Binary)> {
    // in theory we've already instantiated the contract so this cannot fail...
    let counter = counter_in.clone() + 1;
    let new_hash = gen_hash(deps, counter, current_hash);
    // return the the counter,
    Ok((counter, new_hash))
}
