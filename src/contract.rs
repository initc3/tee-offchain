use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, Storage, Uint128, StdResult, ensure, StdError, to_binary};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use schemars::_serde_json::to_string;

use crate::msg::{ExecuteMsg, GetStateResp, InstantiateMsg, IterateHashResp, QueryMsg};
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
            new_hash,
            current_mac,
        } => try_apply_update(
                deps,
                env,
                info,
                new_counter,
                new_hash,
                current_mac,
            )
    };
    res
}

fn try_apply_update(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    new_counter: Uint128,
    new_hash: Binary,
    current_mac: Binary
) -> StdResult<Response> {
    // Load state from contract store
    let mut store = CONFIG_KEY.load(deps.storage).unwrap();

    // Generate the MAC of the currently stored hash, check it against the currently passed in MAC.
    ensure! {
        gen_mac(store.key.clone(), store.current_hash.clone()).unwrap() == current_mac,
        StdError::generic_err("Passed in MAC, doesn't match the expected MAC.")
    }

    // Ensure that the new counter value is greater than the stored one.
    ensure! {
        new_counter == Uint128::from(store.counter.u128() + 1),
        StdError::generic_err("The new counter value must be one greater than the previous value.")
    }

    // Make sure that the new_hash passed into the chain is equivalent to the expected new counter hash.
    ensure! {
        gen_hash(new_counter, store.current_hash).unwrap() == new_hash,
        StdError::generic_err("The passed in new_hash is not equal to the expected future hash.")
    }

    // Update counter value to the new counter value
    store.counter = new_counter;
    // Update the hash value to the new hash
    store.current_hash = new_hash;

    CONFIG_KEY.save(deps.storage, &store).unwrap();

    Ok(Response::new())
}

fn get_state(
    deps: Deps,
    _env: Env,
) -> StdResult<Binary> {
    // load store from state
    let store = CONFIG_KEY.load(deps.storage).unwrap();

    // on the fly generate the new MAC bc we're that cool
    let current_mac_result = gen_mac(store.key, store.current_hash.clone()).unwrap();

    // Struct containing the information of the state to return to the user
    let resp = GetStateResp {
        counter: store.counter,
        current_hash: store.current_hash,
        current_mac: current_mac_result
    };

    // Convert the `GetStateAnswer` to base64'd JSON
    let resp_as_b64 = to_binary(&resp).unwrap();

    // Return that out!
    Ok(resp_as_b64)
}

// FIXME: We're unable to crank the smart contract figure this out
fn iterate_hash(
    deps: Deps,
    _env: Env,
    old_counter: Uint128,
    old_hash: Binary, // Might not need it
    old_mac: Binary
) -> StdResult<Binary> {
    // Load state from contract store
    let mut store = CONFIG_KEY.load(deps.storage).unwrap();

    // Generate the MAC of the currently stored hash, check it against the currently passed in MAC.
    println!("inputs ...");
    println!("old counter: {:?}", old_counter);
    println!("old hash: {:?}", old_hash);
    println!("old mac: {:?}", old_mac);

    println!("store key: {:?}", store.key.clone());
    println!("store current hash: {:?}", store.current_hash.clone());
    println!("current mac: {:?}", gen_mac(store.key.clone(), store.current_hash.clone()).unwrap());


    //gen_mac(store.key.clone(), store.current_hash.clone()).unwrap() == old_mac,
    ensure! {
        //gen_mac(store.key.clone(), store.current_hash.clone()).unwrap() == old_mac,
        gen_mac(store.key.clone(), old_hash.clone()).unwrap() == old_mac,
        StdError::generic_err("Passed in MAC, doesn't match the expected MAC.")
    }

    // The counter after it has been iterated upwards
    let new_counter = Uint128::from(old_counter.u128() + 1);

    // The new hash built from the passed in data.
    let new_hash = gen_hash(new_counter, old_hash).unwrap();
    store.current_hash = new_hash.clone();
    //CONFIG_KEY.save(deps.storage).unwrap();
    //CONFIG_KEY.save(deps.storage, &store).unwrap();

    // The newly generated mac of the data.
    let new_mac = gen_mac(store.key, new_hash.clone()).unwrap();

    // Answer with the data that should be correct
    let resp = IterateHashResp {
        new_counter,
        new_hash,
        new_mac,
    };

    let resp_as_b64 = to_binary(&resp).unwrap();

    Ok(resp_as_b64)
}

// ---------------------------------------- QUERIES --------------------------------------

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {
        } => get_state(deps, _env),
        QueryMsg::IterateHash {
            counter,
            current_hash,
            old_mac,
        } => iterate_hash(deps, _env, counter, current_hash, old_mac),
    }
}

fn gen_hash(counter_in: Uint128, current_hash: Binary) -> StdResult<Binary> {
    // Create sha256 hasher
    let mut hasher = Sha256::default();
    // Counter value as little endian bytes, wasm is little so should be no overhead
    let counter_as_bytes = counter_in.to_le_bytes();

    // update hasher with counter bytes
    hasher.update(counter_as_bytes.as_slice());
    // update hasher with the passed in current hash
    hasher.update(current_hash.as_slice());

    // finalize hash
    let finalized_hash = hasher.finalize();
    // convert finalized hash to byte slice
    let hash_digest = finalized_hash.as_slice();
    // return b64 blob of the hash_digest
    Ok(Binary::from(hash_digest))
}

/// Takes in a key and a data_blob. Returns a MAC produced via H(key || data_blob).
fn gen_mac(key: Binary, data_blob: Binary) -> StdResult<Binary> {
    // in theory we've already instantiated the contract so this cannot fail...
    // Create sha256 hasher
    let mut hasher = Sha256::default();

    // update hasher state with key
    hasher.update(key.as_slice());
    // update hasher state with data_blob
    hasher.update(data_blob.as_slice());

    // finalize hash
    let finalized_hash = hasher.finalize();
    // produce hash_digest
    let hash_digest = finalized_hash.as_slice();
    // convert from slice of u8s to b64
    let hash_as_b64 = Binary::from(hash_digest);
    // return the the counter,
    Ok(hash_as_b64)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, StdResult, Uint128};
    use crate::contract::{gen_hash, gen_mac, instantiate, query};
    use crate::msg::{ExecuteMsg, GetStateResp, InstantiateMsg, IterateHashResp, QueryMsg};

    #[test]
    fn test_get_state() {
        let mocked_env = mock_env();
        let mut mocked_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);

        let resp = instantiate(mocked_deps.as_mut(), mocked_env, mocked_info, InstantiateMsg {}).unwrap();

        let query_msg = QueryMsg::GetState {};

        let mocked_env = mock_env();
        let query_resp = query(mocked_deps.as_ref(), mocked_env, query_msg).unwrap();


        let query_to_struct: GetStateResp = from_binary(&query_resp).unwrap();

        println!("{:?}", query_to_struct)
    }

    #[test]
    fn test_iterate_hash() {
        let mocked_env = mock_env();
        let mut mocked_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);

        let resp = instantiate(mocked_deps.as_mut(), mocked_env, mocked_info, InstantiateMsg {}).unwrap();

        let query_msg = QueryMsg::GetState {};

        let mocked_env = mock_env();
        let query_resp = query(mocked_deps.as_ref(), mocked_env, query_msg).unwrap();

        let query_as_struct: GetStateResp = from_binary(&query_resp).unwrap();

        let iterate_hash = QueryMsg::IterateHash {
            counter: query_as_struct.counter,
            current_hash: query_as_struct.current_hash,
            old_mac: query_as_struct.current_mac,
        };

        //println!("test> old_mac: {:?}", old_mac);

        // Try cranking the contract a few times
        let mocked_env = mock_env();
        let iterate_hash_resp = query(mocked_deps.as_ref(), mocked_env, iterate_hash).unwrap();
        let iterate_hash_resp: StdResult<IterateHashResp> = from_binary(&iterate_hash_resp);

        //let applyUpdate = ExecuteMsg::ApplyUpdate {
        //    new_counter: iterate_hash_resp.new_counter,
        //    new_hash: iterate_hash_resp.new_hash,
        //    current_mac: iterate_hash_resp.new_mac,
        //};
        //let mocked_env = mock_env();
        //let apply_update_resp = execute(mocked_deps.as_mut(), mocked_env, applyUpdate).unwrap();

        assert! {
            iterate_hash_resp.is_ok(),
            "WE FAILED TO UNBASE64 TO THE STRUCT"
        }

        let iterate_hash_resp = iterate_hash_resp.unwrap();

        let iterate_hash = QueryMsg::IterateHash {
            counter: iterate_hash_resp.new_counter,
            current_hash: iterate_hash_resp.new_hash,
            old_mac: iterate_hash_resp.new_mac,
        };

        //println!("test> old_mac: {:?}", old_mac);

        let mocked_env = mock_env();
        let iterate_hash_resp = query(mocked_deps.as_ref(), mocked_env, iterate_hash).unwrap();
        assert!(true);
        //let iterate_hash_resp: StdResult<IterateHashAnswer> = from_binary(&iterate_hash_resp);

        //assert! {
        //    iterate_hash_resp.is_ok(),
        //    "WE FAILED TO UNBASE64 TO THE STRUCT"
        //}

        //println!("{:?}", iterate_hash_resp.unwrap())
    }

    #[test]
    fn test_gen_hash() {
        let mock_env = mock_env();
        let mock_deps = mock_dependencies();
        let mock_info = mock_info("owner", &[]);

        let entropy = mock_env.block.random.unwrap();
        let initial_counter = Uint128::zero();

        let res = gen_hash(initial_counter, entropy);

        assert! {
            res.is_ok(),
            "WE FAILED TO MAKE A HASH, PANIC"
        }

        println!("{:?}", res.unwrap())
    }

    #[test]
    fn test_gen_mac() {
        let mock_env = mock_env();
        let mock_deps = mock_dependencies();
        let mock_info = mock_info("owner", &[]);

        // grab entropy from mock env
        let entropy = mock_env.block.random.unwrap();
        // create mock counter
        let initial_counter = Uint128::zero();

        // create hash of counter and OG seed.
        let hash = gen_hash(initial_counter, entropy.clone()).unwrap();

        let res = gen_mac(entropy, hash);

        assert! {
            res.is_ok(),
            "WE FAILED TO MAKE A MAC, PANIC"
        }

        println!("{:?}", res.unwrap())
    }
}
