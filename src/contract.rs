use cosmwasm_std::{Coin, entry_point, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint128, StdResult, ensure, StdError, to_binary, Addr, CosmosMsg};
use sha2::{Sha256, Digest};
use crate::msg::{ExecuteMsg, GetStateAnswer, InstantiateMsg, IterateHashAnswer, QueryMsg, GetRequestAnswer, ProcessResponseAnswer};
use crate::state::{State, ReqType, CheckPoint, Request, ResponseState, AddressBalance};
use crate::state::{CHECKPOINT_KEY, PREFIX_REQUESTS_KEY, CONFIG_KEY, REQUEST_SEQNO_KEY, AEAD_KEY, REQUEST_LEN_KEY};
use crate::utils::{get_key, bool_to_uint128};
use cosmwasm_std::ReplyOn::Success;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    // grab random entropy that is produced by the consensus
    let entropy = env.block.random.as_ref().unwrap();

    // The `State` is created
    let config = State {
        owner: info.sender,
        key: entropy.clone(),
        current_hash: entropy.clone(),
        counter: Uint128::zero()
    };

    let zero_val = Uint128::zero();

    let symmetric_key = get_key(env);
    let checkpoint = CheckPoint {
        checkpoint: Vec::new(),
        seqno: zero_val,
        resp_seqno: zero_val
    };

    // Save data to storage
    CONFIG_KEY.save(deps.storage, &config).unwrap();
    REQUEST_SEQNO_KEY.save(deps.storage, &zero_val).unwrap();
    REQUEST_LEN_KEY.save(deps.storage, &zero_val).unwrap();
    CHECKPOINT_KEY.save(deps.storage, &checkpoint).unwrap();
    AEAD_KEY.save(deps.storage, &symmetric_key).unwrap();

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
            ),
        ExecuteMsg::SubmitDeposit {
        } => try_submit_deposit(
                deps,
                env,
                info,
            ),
        ExecuteMsg::SubmitTransfer {
            to,
            amount,
            memo
        } => try_submit_transfer(
                deps,
                env,
                info,
                to,
                amount,
                memo
            ),
        ExecuteMsg::SubmitWithdraw {
            amount
        } => try_submit_withdraw(
                deps,
                env,
                info,
                amount
            ),
        ExecuteMsg::CommitResponse {
            cipher
        } => try_commit_response(
                deps,
                env,
                info,
                cipher
            ),
        ExecuteMsg::WriteCheckpoint {
            cipher
        } => try_write_checkpoint(
                deps,
                env,
                info,
                cipher
        ),
        ExecuteMsg::CreateViewingKey { 
            entropy
        } => try_create_key(
            deps, 
            env, 
            info, 
            entropy
        ),
        ExecuteMsg::SetViewingKey { 
            key
        } => try_set_key(
            deps, 
            info, 
            key
        ),
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
    //TODO add event
    Ok(Response::new())
}

fn try_submit_deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> StdResult<Response> {

    let mut amount = Uint128::zero();

    for coin in &info.funds {
        amount += coin.amount
    }

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be deposited"));
    }

    let request = Request {
        reqtype: ReqType::DEPOSIT,
        from: info.sender,
        to: None,
        amount: amount,
        memo: None
    };
    let req_len = REQUEST_LEN_KEY.load(deps.storage).unwrap();
    let new_len = req_len.checked_add(Uint128::one()).unwrap();
    REQUEST_LEN_KEY.save(deps.storage, &new_len).unwrap();
    println!("try_submit_deposit save at seqno {:?}", req_len);
    Request::save(deps.storage, request, req_len).unwrap();
    //TODO add event
    Ok(Response::default())
}

fn try_submit_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    to: Addr,
    amount: Uint128,
    memo: String
) -> StdResult<Response> {

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be transfered"));
    }
    //TODO save amount in contract

    let request = Request {
        reqtype: ReqType::TRANSFER,
        from: info.sender,
        to: Some(to),
        amount: amount,
        memo: Some(memo)
    };
    let req_len = REQUEST_LEN_KEY.load(deps.storage).unwrap();
    let new_len = req_len.checked_add(Uint128::one()).unwrap();
    REQUEST_LEN_KEY.save(deps.storage, &new_len).unwrap();
    println!("try_submit_transfer save at seqno {:?}", new_len);
    Request::save(deps.storage, request, req_len).unwrap();
    //TODO add event
    Ok(Response::default())
}

fn try_submit_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be transfered"));
    }

    let request = Request {
        reqtype: ReqType::WITHDRAW,
        from: info.sender,
        to: None,
        amount: amount,
        memo: None
    };
    let req_len = REQUEST_LEN_KEY.load(deps.storage).unwrap();
    let new_len = req_len.checked_add(Uint128::one()).unwrap();
    REQUEST_LEN_KEY.save(deps.storage, &new_len).unwrap();
    println!("try_submit_withdraw save at seqno {:?}", new_len);
    Request::save(deps.storage, request, req_len).unwrap();
    Ok(Response::default())
}

fn try_commit_response(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    cipher: Binary,
) -> StdResult<Response> {

    let seqno = REQUEST_SEQNO_KEY.load(deps.storage).unwrap();
    let req_len = REQUEST_LEN_KEY.load(deps.storage).unwrap();

    let response = ResponseState::decrypt_response(deps.storage, cipher).unwrap();
    // println!("try_commit_response seqno {:?} req_seqno {:?}", response.seqno, seqno);
    if  response.seqno != seqno {
        return Err(StdError::generic_err("Response should processes strictly in order"));
    }
    // println!("try_commit_response response.seqno {:?} < req_len {:?}", response.seqno, req_len);
    if  response.seqno >= req_len {
        return Err(StdError::generic_err("Response seqno less than number of requests"));
    }
    let new_seqno = response.seqno.checked_add(Uint128::one()).unwrap();
    println!("try_commit_response update seqno to {:?}", new_seqno);

    REQUEST_SEQNO_KEY.save(deps.storage, &new_seqno).unwrap();

    println!("try_commit_response load at seqno {:?}", response.seqno);

    let request = Request::load(deps.storage, response.seqno).unwrap();
    if request.reqtype == ReqType::WITHDRAW {
        let withdrawal_coins: Vec<Coin> = vec![Coin {
            denom: "uscrt".to_string(),
            amount: response.amount,
        }];
        let message: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
            to_address: request.from.clone().into_string(),
            amount: withdrawal_coins,
        });
        println!("transfer message {:?}", message);
    }
    //todo emit event
    Ok(Response::default())
}

fn try_write_checkpoint(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    cipher: Binary,
) -> StdResult<Response> {
    let new_checkpoint: CheckPoint = CheckPoint::decrypt_checkpoint(deps.storage, cipher).unwrap();
    let old_checkpoint: CheckPoint = CheckPoint::load(deps.storage).unwrap();

    if old_checkpoint.seqno > new_checkpoint.seqno {
        return Err(StdError::generic_err("New Checkpoint Seq no too low"));
    }
    println!("try_write_checkpoint {:?}", new_checkpoint);

    CheckPoint::save(deps.storage, new_checkpoint).unwrap();


    Ok(Response::default())
}

pub fn try_create_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let key = ViewingKey::create(
        deps.storage,
        &info,
        &env,
        info.sender.as_str(),
        entropy.as_ref(),
    );
    Ok(Response::new().set_data(to_binary(&key)?))
}

pub fn try_set_key(deps: DepsMut, info: MessageInfo, key: String) -> StdResult<Response> {
    ViewingKey::set(deps.storage, info.sender.as_str(), key.as_str());
    Ok(Response::default())
}

// ---------------------------------------- QUERIES --------------------------------------

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {
        } => qet_state(deps, env),
        QueryMsg::IterateHash {
            counter,
            current_hash,
            old_mac,
        } => iterate_hash(deps, env, counter, current_hash, old_mac),
        QueryMsg::GetRequest {
            seqno,
        } => get_request(deps, env, seqno),
        QueryMsg::GetCheckpoint {
        } => get_checkpoint(deps, env),
        QueryMsg::ProcessNext {
            cipher
        } => process_request(deps, env, cipher),
        QueryMsg::GetBalance {
            address,
            key
        } => get_balance(deps, env, address, key)
    }
}

fn qet_state(
    deps: Deps,
    _env: Env,
) -> StdResult<Binary> {
    // load store from state
    let store = CONFIG_KEY.load(deps.storage).unwrap();

    // on the fly generate the new MAC bc we're that cool
    let current_mac_result = gen_mac(store.key, store.current_hash.clone()).unwrap();

    // Struct containing the information of the state to return to the user
    let resp = GetStateAnswer {
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
    let resp = IterateHashAnswer {
        new_counter,
        new_hash,
        new_mac,
    };

    let resp_as_b64 = to_binary(&resp).unwrap();

    Ok(resp_as_b64)
}


fn get_request(
    deps: Deps,
    _env: Env,
    seqno: Uint128,
) -> StdResult<Binary> {
    let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());
    let request = req_key.load(deps.storage).unwrap();

    let resp = GetRequestAnswer {
        reqtype: request.reqtype,
        from: request.from
    };

    // Convert the `GetStateAnswer` to base64'd JSON
    let resp_as_b64 = to_binary(&resp).unwrap();

    // Return that out!
    Ok(resp_as_b64)
}

fn get_checkpoint(
    deps: Deps,
    env: Env
) -> StdResult<Binary> {

    let checkpoint = CheckPoint::load(deps.storage)?;
    let cipher = CheckPoint::encrypt_checkpoint(deps.storage, env, checkpoint)?;

    let resp_as_b64 = to_binary(&cipher).unwrap();

    Ok(resp_as_b64)
}

fn get_balance(
    deps: Deps,
    _env: Env,
    address: Addr,
    key: String
) -> StdResult<Binary> {
    let result = ViewingKey::check(deps.storage, address.as_str(), key.as_str());
    if result.is_err() {
        return Err(StdError::generic_err("Viewing key is incorrect"));
    }

    let checkpoint = CheckPoint::load(deps.storage)?;
    // println!("get_balance checkpoint {:?}", checkpoint);
    let mut res = Uint128::zero();
    for i in 0..checkpoint.checkpoint.len() {
        let a = checkpoint.checkpoint.get(i).unwrap();
        let b: bool = a.address == address;
        let b_int = bool_to_uint128(b);
        res = res.checked_add(a.balance.checked_mul(b_int).unwrap()).unwrap();
    }
    let resp_as_b64 = to_binary(&res).unwrap();
    Ok(resp_as_b64)
}

fn process_request(
    deps: Deps,
    env: Env,
    cipher: Binary
)-> StdResult<Binary> {
    let mut checkpoint: CheckPoint = CheckPoint::decrypt_checkpoint(deps.storage, cipher).unwrap();
    let seqno = checkpoint.resp_seqno;
    println!("process_request seqno {:?}", seqno);
    let request = Request::load(deps.storage, seqno).unwrap();
    let mut found: bool = false;
    for i in 0..checkpoint.checkpoint.len() {
        let a = checkpoint.checkpoint.get_mut(i).unwrap();
        if a.address == request.from {
            found = true;
        }
    }
    if !found {
        let a = AddressBalance{balance: Uint128::zero(), address: request.from.clone()};
        checkpoint.checkpoint.push(a);
    }
    let response = match request.reqtype {
        ReqType::DEPOSIT {} => {
            for i in 0..checkpoint.checkpoint.len() {
                let a = checkpoint.checkpoint.get_mut(i).unwrap();
                let b: bool = a.address == request.from;
                let b_int = bool_to_uint128(b);
                let m = request.amount.checked_mul(b_int).unwrap();
                checkpoint.checkpoint[i].balance = a.balance.checked_add(m).unwrap();
            }
            ResponseState {
                seqno: seqno,
                status: true,
                amount: Uint128::zero(),
                response: String::default()
            }
        },
        ReqType::WITHDRAW {} => {
            let mut balance_ok: bool = true;
            for i in 0..checkpoint.checkpoint.len() {
                let a = checkpoint.checkpoint.get_mut(i).unwrap();
                let b: bool = a.address == request.from;
                balance_ok = balance_ok && (!b || a.balance >= request.amount);
                let b_int = bool_to_uint128(b);
                let balance_ok_int = bool_to_uint128(balance_ok);
                let m = request.amount.checked_mul(b_int.checked_mul(balance_ok_int).unwrap()).unwrap();
                checkpoint.checkpoint[i].balance = a.balance.checked_sub(m).unwrap();
            }
            let balance_ok_int = bool_to_uint128(balance_ok);
            ResponseState {
                seqno: seqno,
                status: balance_ok,
                amount: request.amount.checked_mul(balance_ok_int).unwrap(),
                response: String::default()
            }
        },
        ReqType::TRANSFER => {
            let mut balance_ok: bool = true;
            for i in 0..checkpoint.checkpoint.len() {
                let a = checkpoint.checkpoint.get_mut(i).unwrap();
                let b: bool = a.address == request.from;
                balance_ok = balance_ok && (!b || a.balance >= request.amount);
                let b_int = bool_to_uint128(b);
                let balance_ok_int = bool_to_uint128(balance_ok);
                let m = request.amount.checked_mul(b_int.checked_mul(balance_ok_int).unwrap()).unwrap();
                checkpoint.checkpoint[i].balance = a.balance.checked_sub(m).unwrap();
            }
            let balance_ok_int = bool_to_uint128(balance_ok);
            for i in 0..checkpoint.checkpoint.len() {
                let a = checkpoint.checkpoint.get_mut(i).unwrap();
                let b: bool = a.address == request.to.clone().unwrap();
                let b_int = bool_to_uint128(b);
                let m = request.amount.checked_mul(b_int.checked_mul(balance_ok_int).unwrap()).unwrap();
                checkpoint.checkpoint[i].balance = a.balance.checked_add(m).unwrap();
            }
            ResponseState {
                seqno: seqno,
                status: balance_ok,
                amount: Uint128::zero(),
                response: String::from("Transfer ok")
            }
        }
    };
    checkpoint.seqno = checkpoint.seqno.checked_add(Uint128::one()).unwrap();
    checkpoint.resp_seqno = checkpoint.resp_seqno.checked_add(Uint128::one()).unwrap();
    println!("process_request requrning checkpoint {:?}", checkpoint);

    let resp_cipher = ResponseState::encrypt_response(deps.storage, env.clone(), response).unwrap();
    let chkpt_cipher = CheckPoint::encrypt_checkpoint(deps.storage, env, checkpoint).unwrap();
    let resp = ProcessResponseAnswer {
        request_cipher: resp_cipher,
        checkpoint_cipher:  chkpt_cipher
    };
    Ok(to_binary(&resp).unwrap())
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
    use cosmwasm_std::{from_binary, StdResult, Uint128, Coin, QueryResponse, StdError, Binary};
    use crate::contract::{get_checkpoint, gen_hash, gen_mac, instantiate, query, execute};
    use crate::msg::{ExecuteMsg, GetStateAnswer, InstantiateMsg, IterateHashAnswer, QueryMsg, ProcessResponseAnswer};
    use crate::state::{CheckPoint};
    // use std::any::Any;
    // use cosmwasm_std::testing::*;

    #[test]
    fn test_get_state() {
        let mocked_env = mock_env();
        let mut mocked_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);

        let _resp = instantiate(mocked_deps.as_mut(), mocked_env, mocked_info.clone(), InstantiateMsg {}).unwrap();

        let query_msg = QueryMsg::GetState {};

        let mocked_env = mock_env();
        let query_resp = query(mocked_deps.as_ref(), mocked_env, query_msg).unwrap();


        let query_to_struct: GetStateAnswer = from_binary(&query_resp).unwrap();

        println!("{:?}", query_to_struct)
    }

    #[test]
    fn test_iterate_hash() {
        let mocked_env = mock_env();
        let mut mocked_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);

        let _resp = instantiate(mocked_deps.as_mut(), mocked_env, mocked_info.clone(), InstantiateMsg {}).unwrap();

        let query_msg = QueryMsg::GetState {};

        let mocked_env = mock_env();
        let query_resp = query(mocked_deps.as_ref(), mocked_env, query_msg).unwrap();

        let query_as_struct: GetStateAnswer = from_binary(&query_resp).unwrap();

        let iterate_hash = QueryMsg::IterateHash {
            counter: query_as_struct.counter,
            current_hash: query_as_struct.current_hash,
            old_mac: query_as_struct.current_mac,
        };

        //println!("test> old_mac: {:?}", old_mac);

        // Try cranking the contract a few times
        let mocked_env = mock_env();
        let iterate_hash_resp = query(mocked_deps.as_ref(), mocked_env, iterate_hash).unwrap();
        let iterate_hash_resp: StdResult<IterateHashAnswer> = from_binary(&iterate_hash_resp);

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
        let _iterate_hash_resp = query(mocked_deps.as_ref(), mocked_env, iterate_hash).unwrap();
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
        let _mock_deps = mock_dependencies();
        let _mock_info = mock_info("owner", &[]);

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
        let _mock_deps = mock_dependencies();
        let _mock_info = mock_info("owner", &[]);

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

    #[test]
    fn test_deposit_no_funds() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert!{
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);
        assert!{
            get_checkpoint_resp.is_ok(),
            "Get Checkpoint Failed"
        }

        let mock_depositer = mock_info("depositer", &[]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert!{
            request_deposit_resp == Err(StdError::generic_err("No funds were sent to be deposited")),
            "Did not fail with insufficient funds"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.sender, key: vk};
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg);
        assert!{
            get_checkpoint_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        assert!(
            Uint128::zero() == balance,
            "Balance should be zero"
        );
    }

    #[test]
    fn test_vk() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        // let answer: ExecuteAnswer = from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env, mocked_info, set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }
    }

    #[test]
    fn test_deposit() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);
        assert! {
            get_checkpoint_resp.is_ok(),
            "Get Checkpoint Failed"
        }

        let checkpoint_cipher: Binary = from_binary(&get_checkpoint_resp.unwrap()).unwrap();

        let mock_depositer = mock_info("depositer", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000),
        }]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert! {
            request_deposit_resp.is_ok(),
            "Submit Deposit Failed"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.clone().sender, key: vk};
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        // println!("balance {:?}", balance);
        assert!(
            Uint128::zero() == balance,
            "Balance should be 0 before Response Commit"
        );

        let process_next_msg = QueryMsg::ProcessNext{ cipher: checkpoint_cipher };
        let process_next_resp = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg);
        assert!{
            process_next_resp.is_ok(),
            "Process Next Failed"
        }

        let process_answer: ProcessResponseAnswer = from_binary(&process_next_resp.unwrap()).unwrap();
        let commit_response_msg = ExecuteMsg::CommitResponse{cipher: process_answer.request_cipher};
        let commit_response_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg);
        assert! {
            commit_response_resp.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg = ExecuteMsg::WriteCheckpoint{cipher: process_answer.checkpoint_cipher};
        let write_checkpoint_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg);
        assert! {
            write_checkpoint_resp.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_balance_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg);
        assert!{
            get_balance_resp2.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp2.unwrap()).unwrap();
        println!("balance {:?}", balance);
        assert!(
            Uint128::new(1000) == balance,
            "Balance should be 1000 after Response Commit"
        );


    }


    #[test]
    fn test_depose_multi() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);

        let checkpoint_cipher: Binary = from_binary(&get_checkpoint_resp.unwrap().clone()).unwrap();

        let mock_depositer = mock_info("depositer", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000),
        }]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert! {
            request_deposit_resp.is_ok(),
            "Submit Deposit Failed"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.clone().sender, key: vk };
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        assert!(
            Uint128::zero() == balance,
            "Balance should be 0 before Response Commit"
        );

        let mock_depositer2 = mock_info("depositer2", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(2000),
        }]);

        let create_vk_msg2 = ExecuteMsg::CreateViewingKey {
            entropy: "yo2".to_string()
        };
        let create_vk_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), create_vk_msg2);
        assert! {
            create_vk_resp2.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk2: String = from_binary(&create_vk_resp2.unwrap().data.unwrap()).unwrap();
        let set_vk_msg2 = ExecuteMsg::SetViewingKey {
            key: vk2.clone()
        };
        let set_vk_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), set_vk_msg2);
        assert! {
            set_vk_resp2.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg2 = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), request_deposit_msg2);
        assert! {
            request_deposit_resp2.is_ok(),
            "Submit 2nd Deposit Failed"
        }

        let get_balance_msg2 = QueryMsg::GetBalance{ address: mock_depositer2.clone().sender, key: vk2.clone() };
        let get_balance_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp2.is_ok(),
            "Get 2nd Balance Failed"
        }
        let balance2: Uint128 = from_binary(&get_balance_resp2.unwrap()).unwrap();
        // println!("balance {:?}", balance);
        assert!(
            Uint128::zero() == balance2,
            "Balance should be 0 before Response Commit"
        );

        let process_next_msg = QueryMsg::ProcessNext{ cipher: checkpoint_cipher };
        let process_next_resp = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg);
        assert!{
            process_next_resp.is_ok(),
            "Process Next Failed"
        }
        let process_answer: ProcessResponseAnswer = from_binary(&process_next_resp.unwrap()).unwrap();


        let process_next_msg2 = QueryMsg::ProcessNext{ cipher: process_answer.clone().checkpoint_cipher };
        let process_next_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg2);
        assert!{
            process_next_resp2.is_ok(),
            "Process Next Failed"
        }
        let process_answer2: ProcessResponseAnswer = from_binary(&process_next_resp2.unwrap()).unwrap();

        let commit_response_msg = ExecuteMsg::CommitResponse{cipher: process_answer.request_cipher};
        let commit_response_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg);
        assert! {
            commit_response_resp.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg = ExecuteMsg::WriteCheckpoint{cipher: process_answer.checkpoint_cipher};
        let write_checkpoint_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg);
        assert! {
            write_checkpoint_resp.is_ok(),
            "Write Checkpoint Failed"
        }

        // let get_checkpoint_msg2 = QueryMsg::GetCheckpoint{};

        // let get_checkpoint_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg2);
        // let checkpoint_cipher2: Binary = from_binary(&get_checkpoint_resp2.unwrap().clone()).unwrap();

        let commit_response_msg2 = ExecuteMsg::CommitResponse{cipher: process_answer2.request_cipher};
        let commit_response_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg2);
        assert! {
            commit_response_resp2.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg2 = ExecuteMsg::WriteCheckpoint{cipher: process_answer2.checkpoint_cipher};
        let write_checkpoint_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg2);
        assert! {
            write_checkpoint_resp2.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        println!("balance {:?}", balance);
        assert!(
            Uint128::new(1000) == balance,
            "Balance should be 1000 after Response Commit"
        );

        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get Balance Failed"
        }
        let balance2: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        println!("balance {:?}", balance2);
        assert!(
            Uint128::new(2000) == balance2,
            "Balance should be 2000 after Response Commit"
        );


    }

    #[test]
    fn test_withdraw() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);
        assert! {
            get_checkpoint_resp.is_ok(),
            "Get Checkpoint Failed"
        }

        let checkpoint_cipher: Binary = from_binary(&get_checkpoint_resp.unwrap()).unwrap();

        let mock_depositer = mock_info("depositer", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000),
        }]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert! {
            request_deposit_resp.is_ok(),
            "Submit Deposit Failed"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.clone().sender, key: vk.clone() };
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        println!("balance of depositor1 BEFORE COMMIT DEPOSIT {:?}", balance);

        assert!(
            Uint128::zero() == balance,
            "Balance should be 0 before Response Commit"
        );

        let process_next_msg = QueryMsg::ProcessNext{ cipher: checkpoint_cipher };
        let process_next_resp = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg);
        assert!{
            process_next_resp.is_ok(),
            "Process Next Failed"
        }

        let process_answer: ProcessResponseAnswer = from_binary(&process_next_resp.unwrap()).unwrap();
        let commit_response_msg = ExecuteMsg::CommitResponse{cipher: process_answer.request_cipher};
        let commit_response_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg);
        assert! {
            commit_response_resp.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg = ExecuteMsg::WriteCheckpoint{cipher: process_answer.checkpoint_cipher};
        let write_checkpoint_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg);
        assert! {
            write_checkpoint_resp.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_balance_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp2.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp2.unwrap()).unwrap();
        println!("balance of depositor2 BEFORE COMMIT DEPOSIT {:?}", balance);
        assert!(
            Uint128::new(1000) == balance,
            "Balance should be 1000 after Response Commit"
        );

        let request_withdraw_msg = ExecuteMsg::SubmitWithdraw{amount: Uint128::new(500)};
        let request_withdraw_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_withdraw_msg);
        assert! {
            request_withdraw_resp.is_ok(),
            "Submit Withdraw Failed"
        }
        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get Balance Failed"
        }
        let balance3: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT DEPOSIT {:?}", balance3);
        assert!(
            Uint128::new(1000) == balance3,
            "Balance should still be 1000 before Response Commit"
        );


        let get_checkpoint_msg2 = QueryMsg::GetCheckpoint{};

        let get_checkpoint_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg2);
        let checkpoint_cipher2: Binary = from_binary(&get_checkpoint_resp2.unwrap().clone()).unwrap();

        let process_next_msg2 = QueryMsg::ProcessNext{ cipher: checkpoint_cipher2 };
        let process_next_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg2);
        assert!{
            process_next_resp2.is_ok(),
            "Process Next Failed"
        }
        let process_answer2: ProcessResponseAnswer = from_binary(&process_next_resp2.unwrap()).unwrap();

        let commit_response_msg2 = ExecuteMsg::CommitResponse{cipher: process_answer2.request_cipher};
        let commit_response_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg2);
        assert! {
            commit_response_resp2.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg2 = ExecuteMsg::WriteCheckpoint{cipher: process_answer2.checkpoint_cipher};
        let write_checkpoint_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg2);
        assert! {
            write_checkpoint_resp2.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get Balance Failed"
        }
        let balance4: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT WITHDRAW {:?}", balance4);
        assert!(
            Uint128::new(500) == balance4,
            "Balance should still be 500 after Response Commit"
        );

    }

    #[test]
    fn test_withdraw_too_large() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);
        assert! {
            get_checkpoint_resp.is_ok(),
            "Get Checkpoint Failed"
        }

        let checkpoint_cipher: Binary = from_binary(&get_checkpoint_resp.unwrap()).unwrap();

        let mock_depositer = mock_info("depositer", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000),
        }]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert! {
            request_deposit_resp.is_ok(),
            "Submit Deposit Failed"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.clone().sender, key: vk.clone() };
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        println!("balance of depositor1 BEFORE COMMIT DEPOSIT {:?}", balance);

        assert!(
            Uint128::zero() == balance,
            "Balance should be 0 before Response Commit"
        );

        let process_next_msg = QueryMsg::ProcessNext{ cipher: checkpoint_cipher };
        let process_next_resp = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg);
        assert!{
            process_next_resp.is_ok(),
            "Process Next Failed"
        }

        let process_answer: ProcessResponseAnswer = from_binary(&process_next_resp.unwrap()).unwrap();
        let commit_response_msg = ExecuteMsg::CommitResponse{cipher: process_answer.request_cipher};
        let commit_response_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg);
        assert! {
            commit_response_resp.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg = ExecuteMsg::WriteCheckpoint{cipher: process_answer.checkpoint_cipher};
        let write_checkpoint_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg);
        assert! {
            write_checkpoint_resp.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_balance_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp2.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp2.unwrap()).unwrap();
        println!("balance of depositor2 BEFORE COMMIT DEPOSIT {:?}", balance);
        assert!(
            Uint128::new(1000) == balance,
            "Balance should be 1000 after Response Commit"
        );

        let request_withdraw_msg = ExecuteMsg::SubmitWithdraw{amount: Uint128::new(1500)};
        let request_withdraw_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_withdraw_msg);
        assert! {
            request_withdraw_resp.is_ok(),
            "Submit Withdraw Failed"
        }
        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get Balance Failed"
        }
        let balance3: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT DEPOSIT {:?}", balance3);
        assert!(
            Uint128::new(1000) == balance3,
            "Balance should still be 1000 before Response Commit"
        );

        let get_checkpoint_msg2 = QueryMsg::GetCheckpoint{};

        let get_checkpoint_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg2);
        let checkpoint_cipher2: Binary = from_binary(&get_checkpoint_resp2.unwrap().clone()).unwrap();

        let process_next_msg2 = QueryMsg::ProcessNext{ cipher: checkpoint_cipher2 };
        let process_next_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg2);
        assert!{
            process_next_resp2.is_ok(),
            "Process Next Failed"
        }
        let process_answer2: ProcessResponseAnswer = from_binary(&process_next_resp2.unwrap()).unwrap();

        let commit_response_msg2 = ExecuteMsg::CommitResponse{cipher: process_answer2.request_cipher};
        let commit_response_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg2);
        assert! {
            commit_response_resp2.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg2 = ExecuteMsg::WriteCheckpoint{cipher: process_answer2.checkpoint_cipher};
        let write_checkpoint_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg2);
        assert! {
            write_checkpoint_resp2.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get Balance Failed"
        }
        let balance4: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT failed WITHDRAW {:?}", balance4);
        assert!(
            Uint128::new(1000) == balance4,
            "Balance should still be 1000 after Response Commit"
        );

    }

    #[test]
    fn test_transfer() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);

        let checkpoint_cipher: Binary = from_binary(&get_checkpoint_resp.unwrap().clone()).unwrap();

        let mock_depositer = mock_info("depositer", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000),
        }]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert! {
            request_deposit_resp.is_ok(),
            "Submit Deposit Failed"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.clone().sender, key: vk };
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        println!("balance of depositor1 BEFORE COMMIT DEPOSIT {:?}", balance);
        assert!(
            Uint128::zero() == balance,
            "Balance should be 0 before Response Commit"
        );

        let mock_depositer2 = mock_info("depositer2", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(2000),
        }]);

        let create_vk_msg2 = ExecuteMsg::CreateViewingKey {
            entropy: "yo2".to_string()
        };
        let create_vk_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), create_vk_msg2);
        assert! {
            create_vk_resp2.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk2: String = from_binary(&create_vk_resp2.unwrap().data.unwrap()).unwrap();
        let set_vk_msg2 = ExecuteMsg::SetViewingKey {
            key: vk2.clone()
        };
        let set_vk_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), set_vk_msg2);
        assert! {
            set_vk_resp2.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg2 = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), request_deposit_msg2);
        assert! {
            request_deposit_resp2.is_ok(),
            "Submit 2nd Deposit Failed"
        }

        let get_balance_msg2 = QueryMsg::GetBalance{ address: mock_depositer2.clone().sender, key: vk2 };
        let get_balance_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp2.is_ok(),
            "Get 2nd Balance Failed"
        }
        let balance2: Uint128 = from_binary(&get_balance_resp2.unwrap()).unwrap();
        println!("balance of depositor2 BEFORE COMMIT DEPOSIT {:?}", balance2);
        assert!(
            Uint128::zero() == balance2,
            "Balance should be 0 before Response Commit"
        );

        let process_next_msg = QueryMsg::ProcessNext{ cipher: checkpoint_cipher };
        let process_next_resp = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg);
        assert!{
            process_next_resp.is_ok(),
            "Process Next Failed"
        }
        let process_answer: ProcessResponseAnswer = from_binary(&process_next_resp.unwrap()).unwrap();

        let commit_response_msg = ExecuteMsg::CommitResponse{cipher: process_answer.request_cipher};
        let commit_response_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg);
        assert! {
            commit_response_resp.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg = ExecuteMsg::WriteCheckpoint{cipher: process_answer.checkpoint_cipher};
        let write_checkpoint_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg);
        assert! {
            write_checkpoint_resp.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_checkpoint_msg2 = QueryMsg::GetCheckpoint{};

        let get_checkpoint_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg2);
        let checkpoint_cipher2: Binary = from_binary(&get_checkpoint_resp2.unwrap().clone()).unwrap();

        let process_next_msg2 = QueryMsg::ProcessNext{ cipher: checkpoint_cipher2 };
        let process_next_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg2);
        assert!{
            process_next_resp2.is_ok(),
            "Process Next Failed"
        }
        let process_answer2: ProcessResponseAnswer = from_binary(&process_next_resp2.unwrap()).unwrap();

        let commit_response_msg2 = ExecuteMsg::CommitResponse{cipher: process_answer2.request_cipher};
        let commit_response_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg2);
        assert! {
            commit_response_resp2.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg2 = ExecuteMsg::WriteCheckpoint{cipher: process_answer2.checkpoint_cipher};
        let write_checkpoint_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg2);
        assert! {
            write_checkpoint_resp2.is_ok(),
            "Write Checkpoint Failed"
        }
        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT DEPOSIT {:?}", balance);

        assert!(
            Uint128::new(1000) == balance,
            "Balance should be 1000 after Response Commit"
        );

        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get Balance Failed"
        }
        let balance2: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        println!("balance of depositor2 AFTER COMMIT DEPOSIT {:?}", balance2);
        assert!(
            Uint128::new(2000) == balance2,
            "Balance should be 2000 after Response Commit"
        );



        let request_transfer = ExecuteMsg::SubmitTransfer{to: mock_depositer.clone().sender, amount: Uint128::new(500), memo: String::from("hello")};
        let request_transfer_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), request_transfer);
        assert! {
            request_transfer_resp.is_ok(),
            "Submit Transfer Failed"
        }

        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get 3rd Balance Failed"
        }
        let balance3: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        // println!("balance of depositor1 {:?}", balance3);

        assert!(
            Uint128::new(1000) == balance3,
            "Balance should same before Response Commit"
        );


        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get 4th Balance Failed"
        }
        let balance4: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        // println!("balance of depositor2 {:?}", balance4);
        assert!(
            Uint128::new(2000) == balance4,
            "Balance should same before Response Commit"
        );




        let get_checkpoint_msg3 = QueryMsg::GetCheckpoint{};

        let get_checkpoint_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg3);
        let checkpoint_cipher3: Binary = from_binary(&get_checkpoint_resp3.unwrap().clone()).unwrap();

        let process_next_msg3 = QueryMsg::ProcessNext{ cipher: checkpoint_cipher3 };
        let process_next_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg3);
        assert!{
            process_next_resp3.is_ok(),
            "Process Next Failed"
        }
        let process_answer3: ProcessResponseAnswer = from_binary(&process_next_resp3.unwrap()).unwrap();

        let commit_response_msg3 = ExecuteMsg::CommitResponse{cipher: process_answer3.request_cipher};
        let commit_response_resp3 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg3);
        assert! {
            commit_response_resp3.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg3 = ExecuteMsg::WriteCheckpoint{cipher: process_answer3.checkpoint_cipher};
        let write_checkpoint_resp3 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg3);
        assert! {
            write_checkpoint_resp3.is_ok(),
            "Write Checkpoint Failed"
        }


        let get_balance_resp5 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp5.is_ok(),
            "Get 5th Balance Failed"
        }
        let balance5: Uint128 = from_binary(&get_balance_resp5.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT TRANSFER {:?}", balance5);

        assert!(
            Uint128::new(1500) == balance5,
            "Balance should updated after Response Commit"
        );

        let get_balance_resp6 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp6.is_ok(),
            "Get 6th Balance Failed"
        }
        let balance6: Uint128 = from_binary(&get_balance_resp6.unwrap()).unwrap();
        println!("balance of depositor2 AFTER COMMIT TRANSFER  {:?}", balance6);
        assert!(
            Uint128::new(1500) == balance6,
            "Balance should updated after Response Commit"
        );
    }

    #[test]
    fn test_transfer_too_large() {
        let mocked_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);
        let init_resp = instantiate(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), InstantiateMsg {});
        assert! {
            init_resp.is_ok(),
            "Instantiate Failed"
        }

        let get_checkpoint_msg = QueryMsg::GetCheckpoint{};
        let get_checkpoint_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg);

        let checkpoint_cipher: Binary = from_binary(&get_checkpoint_resp.unwrap().clone()).unwrap();

        let mock_depositer = mock_info("depositer", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(1000),
        }]);

        let create_vk_msg = ExecuteMsg::CreateViewingKey {
            entropy: "yo".to_string()
        };
        let create_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), create_vk_msg);
        assert! {
            create_vk_resp.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk: String = from_binary(&create_vk_resp.unwrap().data.unwrap()).unwrap();
        let set_vk_msg = ExecuteMsg::SetViewingKey {
            key: vk.clone()
        };
        let set_vk_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), set_vk_msg);
        assert! {
            set_vk_resp.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer.clone(), request_deposit_msg);
        assert! {
            request_deposit_resp.is_ok(),
            "Submit Deposit Failed"
        }

        let get_balance_msg = QueryMsg::GetBalance{ address: mock_depositer.clone().sender, key: vk };
        let get_balance_resp = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp.unwrap()).unwrap();
        println!("balance of depositor1 BEFORE COMMIT DEPOSIT {:?}", balance);
        assert!(
            Uint128::zero() == balance,
            "Balance should be 0 before Response Commit"
        );

        let mock_depositer2 = mock_info("depositer2", &[Coin {
            denom: "uscrt".to_string(),
            amount: Uint128::new(2000),
        }]);

        let create_vk_msg2 = ExecuteMsg::CreateViewingKey {
            entropy: "yo2".to_string()
        };
        let create_vk_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), create_vk_msg2);
        assert! {
            create_vk_resp2.is_ok(),
            "Creake Viewing Key Failed"
        }
        let vk2: String = from_binary(&create_vk_resp2.unwrap().data.unwrap()).unwrap();
        let set_vk_msg2 = ExecuteMsg::SetViewingKey {
            key: vk2.clone()
        };
        let set_vk_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), set_vk_msg2);
        assert! {
            set_vk_resp2.is_ok(),
            "Set Viewing Key Failed"
        }

        let request_deposit_msg2 = ExecuteMsg::SubmitDeposit{};
        let request_deposit_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), request_deposit_msg2);
        assert! {
            request_deposit_resp2.is_ok(),
            "Submit 2nd Deposit Failed"
        }

        let get_balance_msg2 = QueryMsg::GetBalance{ address: mock_depositer2.clone().sender, key: vk2 };
        let get_balance_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp2.is_ok(),
            "Get 2nd Balance Failed"
        }
        let balance2: Uint128 = from_binary(&get_balance_resp2.unwrap()).unwrap();
        println!("balance of depositor2 BEFORE COMMIT DEPOSIT {:?}", balance2);
        assert!(
            Uint128::zero() == balance2,
            "Balance should be 0 before Response Commit"
        );

        let process_next_msg = QueryMsg::ProcessNext{ cipher: checkpoint_cipher };
        let process_next_resp = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg);
        assert!{
            process_next_resp.is_ok(),
            "Process Next Failed"
        }
        let process_answer: ProcessResponseAnswer = from_binary(&process_next_resp.unwrap()).unwrap();

        let commit_response_msg = ExecuteMsg::CommitResponse{cipher: process_answer.request_cipher};
        let commit_response_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg);
        assert! {
            commit_response_resp.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg = ExecuteMsg::WriteCheckpoint{cipher: process_answer.checkpoint_cipher};
        let write_checkpoint_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg);
        assert! {
            write_checkpoint_resp.is_ok(),
            "Write Checkpoint Failed"
        }

        let get_checkpoint_msg2 = QueryMsg::GetCheckpoint{};

        let get_checkpoint_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg2);
        let checkpoint_cipher2: Binary = from_binary(&get_checkpoint_resp2.unwrap().clone()).unwrap();

        let process_next_msg2 = QueryMsg::ProcessNext{ cipher: checkpoint_cipher2 };
        let process_next_resp2 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg2);
        assert!{
            process_next_resp2.is_ok(),
            "Process Next Failed"
        }
        let process_answer2: ProcessResponseAnswer = from_binary(&process_next_resp2.unwrap()).unwrap();

        let commit_response_msg2 = ExecuteMsg::CommitResponse{cipher: process_answer2.request_cipher};
        let commit_response_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg2);
        assert! {
            commit_response_resp2.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg2 = ExecuteMsg::WriteCheckpoint{cipher: process_answer2.checkpoint_cipher};
        let write_checkpoint_resp2 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg2);
        assert! {
            write_checkpoint_resp2.is_ok(),
            "Write Checkpoint Failed"
        }
        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get Balance Failed"
        }
        let balance: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT DEPOSIT {:?}", balance);

        assert!(
            Uint128::new(1000) == balance,
            "Balance should be 1000 after Response Commit"
        );

        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get Balance Failed"
        }
        let balance2: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        println!("balance of depositor2 AFTER COMMIT DEPOSIT {:?}", balance2);
        assert!(
            Uint128::new(2000) == balance2,
            "Balance should be 2000 after Response Commit"
        );

        let request_transfer = ExecuteMsg::SubmitTransfer{to: mock_depositer.clone().sender, amount: Uint128::new(3000), memo: String::from("hello")};
        let request_transfer_resp = execute(mock_deps.as_mut(), mocked_env.clone(), mock_depositer2.clone(), request_transfer);
        assert! {
            request_transfer_resp.is_ok(),
            "Submit Transfer Failed"
        }

        let get_balance_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp3.is_ok(),
            "Get 3rd Balance Failed"
        }
        let balance3: Uint128 = from_binary(&get_balance_resp3.unwrap()).unwrap();
        // println!("balance of depositor1 {:?}", balance3);

        assert!(
            Uint128::new(1000) == balance3,
            "Balance should same before Response Commit"
        );


        let get_balance_resp4 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp4.is_ok(),
            "Get 4th Balance Failed"
        }
        let balance4: Uint128 = from_binary(&get_balance_resp4.unwrap()).unwrap();
        // println!("balance of depositor2 {:?}", balance4);
        assert!(
            Uint128::new(2000) == balance4,
            "Balance should same before Response Commit"
        );


        let get_checkpoint_msg3 = QueryMsg::GetCheckpoint{};

        let get_checkpoint_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), get_checkpoint_msg3);
        let checkpoint_cipher3: Binary = from_binary(&get_checkpoint_resp3.unwrap().clone()).unwrap();

        let process_next_msg3 = QueryMsg::ProcessNext{ cipher: checkpoint_cipher3 };
        let process_next_resp3 = query(mock_deps.as_ref(), mocked_env.clone(), process_next_msg3);
        assert!{
            process_next_resp3.is_ok(),
            "Process Next Failed"
        }
        let process_answer3: ProcessResponseAnswer = from_binary(&process_next_resp3.unwrap()).unwrap();

        let commit_response_msg3 = ExecuteMsg::CommitResponse{cipher: process_answer3.request_cipher};
        let commit_response_resp3 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), commit_response_msg3);
        assert! {
            commit_response_resp3.is_ok(),
            "Commit Response Failed"
        }

        let write_checkpoint_msg3 = ExecuteMsg::WriteCheckpoint{cipher: process_answer3.checkpoint_cipher};
        let write_checkpoint_resp3 = execute(mock_deps.as_mut(), mocked_env.clone(), mocked_info.clone(), write_checkpoint_msg3);
        assert! {
            write_checkpoint_resp3.is_ok(),
            "Write Checkpoint Failed"
        }


        let get_balance_resp5 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg.clone());
        assert!{
            get_balance_resp5.is_ok(),
            "Get 5th Balance Failed"
        }
        let balance5: Uint128 = from_binary(&get_balance_resp5.unwrap()).unwrap();
        println!("balance of depositor1 AFTER COMMIT failed TRANSFER {:?}", balance5);

        assert!(
            Uint128::new(1000) == balance5,
            "Balance should not updated after Response Commit"
        );

        let get_balance_resp6 = query(mock_deps.as_ref(), mocked_env.clone(), get_balance_msg2.clone());
        assert!{
            get_balance_resp6.is_ok(),
            "Get 6th Balance Failed"
        }
        let balance6: Uint128 = from_binary(&get_balance_resp6.unwrap()).unwrap();
        println!("balance of depositor2 AFTER COMMIT failed TRANSFER  {:?}", balance6);
        assert!(
            Uint128::new(2000) == balance6,
            "Balance should not updated after Response Commit"
        );
    }
}
