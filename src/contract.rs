use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, Storage, Uint128, StdResult, ensure, StdError, to_binary, from_binary, Addr};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use schemars::_serde_json::to_string;
use ring::{aead, error};
use secret_toolkit_crypto::ContractPrng;
use crate::msg::{ExecuteMsg, GetStateAnswer, InstantiateMsg, IterateHashAnswer, QueryMsg, GetRequestAnswer};
use crate::state::{State, ReqType, ReqState, SymmetricKey, CheckPoint, AEAD_KEY, CONFIG_KEY, REQUEST_SEQNO_KEY, PREFIX_REQUESTS_KEY, CHECKPOINT_SEQNO_KEY, PREFIX_CHECKPOINT_KEY, CHECKPOINT_LEN_KEY};

type HmacSha256 = Hmac<Sha256>;
static AES_MODE: &aead::Algorithm = &aead::AES_256_GCM;

/// The IV key byte size
const IV_SIZE: usize = 96/8;
/// Type alias for the IV byte array
type IV = [u8; IV_SIZE];

/// `OneNonceSequence` is a Generic Nonce sequence
pub struct OneNonceSequence(Option<aead::Nonce>);

impl OneNonceSequence {
    /// Constructs the sequence allowing `advance()` to be called
    /// `allowed_invocations` times.
    fn new(nonce: aead::Nonce) -> Self {
        Self(Some(nonce))
    }
}

impl aead::NonceSequence for OneNonceSequence {
    fn advance(&mut self) -> Result<aead::Nonce, error::Unspecified> {
        self.0.take().ok_or(error::Unspecified)
    }
}
pub fn encrypt_with_nonce(message: &[u8], key: &SymmetricKey, iv: &[u8; 12]) -> Result<Vec<u8>, StdError> {

    let aes_encrypt = aead::UnboundKey::new(&AES_MODE, key)
        .map_err(|_| StdError::generic_err("Encryption Error"))?;

    let mut in_out = message.to_owned();
    let tag_size = AES_MODE.tag_len();
    in_out.extend(vec![0u8; tag_size]);
    let _seal_size = {
        let iv = aead::Nonce::assume_unique_for_key(*iv);
        let nonce_sequence = OneNonceSequence::new(iv);
        let mut seal_key: aead::SealingKey<OneNonceSequence> = aead::BoundKey::new(aes_encrypt, nonce_sequence);
        seal_key.seal_in_place_append_tag(aead::Aad::empty(), &mut in_out)
            .map_err(|_| StdError::generic_err("Encryption Error"))
    }?;
    // in_out.truncate(seal_size);
    in_out.extend_from_slice(iv);
    Ok(in_out)
}
pub fn decrypt(cipheriv: &[u8], key: &SymmetricKey) -> Result<Vec<u8>, StdError> {
    if cipheriv.len() < IV_SIZE {
        return Err(StdError::generic_err("Improper encryption Error"));
    }
    let aes_decrypt = aead::UnboundKey::new(&AES_MODE, key)
        .map_err(|_| StdError::generic_err("Unbound key Error"))?;

    let (ciphertext, iv) = cipheriv.split_at(cipheriv.len()-12);
    let nonce = aead::Nonce::try_assume_unique_for_key(&iv).unwrap(); // This Cannot fail because split_at promises that iv.len()==12
    let nonce_sequence = OneNonceSequence::new(nonce);
    let mut ciphertext = ciphertext.to_owned();
    let mut open_key: aead::OpeningKey<OneNonceSequence>  = aead::BoundKey::new(aes_decrypt, nonce_sequence);
    let decrypted_data = match open_key.open_in_place(aead::Aad::empty(), &mut ciphertext) {
        Ok(x) => x,
        Err(e) => {
            println!("symmetric:decrypt open_in_place err {:?}",e);
            return Err(StdError::generic_err("Decryption Error"));
        }
    };
    // let decrypted_data = decrypted_data.map_err(|_| CryptoError::DecryptionError)?;

    Ok(decrypted_data.to_vec())
}

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
        current_hash: entropy.clone(),
        counter: Uint128::zero()
    };
    let mut hasher = Sha256::default();
    hasher.update(entropy.clone().as_slice());
    let finalized_hash = hasher.finalize();
    let prng_seed = finalized_hash.as_slice();
    let mut rng = ContractPrng::new(&prng_seed, entropy.as_slice());
    let symmetric_key: SymmetricKey = rng.rand_bytes();
    let zero_val = Uint128::zero();

    // Save data to storage
    CONFIG_KEY.save(deps.storage, &config).unwrap();
    CHECKPOINT_SEQNO_KEY.save(deps.storage, &zero_val).unwrap();
    REQUEST_SEQNO_KEY.save(deps.storage, &zero_val).unwrap();
    CHECKPOINT_LEN_KEY.save(deps.storage, &zero_val).unwrap();
    REQUEST_SEQNO_KEY.save(deps.storage, &zero_val).unwrap();
    AEAD_KEY.save(deps.storage, &symmetric_key).unwrap();

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
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {

    let mut amount = Uint128::zero();

    for coin in &info.funds {
        amount += coin.amount
    }

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be deposited"));
    }

    // let mut raw_amount = amount.u128();

    let mut seqno = REQUEST_SEQNO_KEY.load(deps.storage).unwrap();
    seqno.checked_add(Uint128::one());

    REQUEST_SEQNO_KEY.save(deps.storage, &seqno).unwrap();

    let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());

    let request = ReqState {
        reqtype: ReqType::DEPOSIT,
        from: info.sender,
        to: None,
        amount: amount,
        memo: None
    };
    req_key.save(deps.storage, &request).unwrap();
    //TODO add event
    Ok(Response::default())
}

fn try_submit_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: Addr,
    amount: Uint128,
    memo: String
) -> StdResult<Response> {

    let mut amount = Uint128::zero();

    for coin in &info.funds {
        amount += coin.amount
    }

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be transfered"));
    }

    let mut seqno = REQUEST_SEQNO_KEY.load(deps.storage).unwrap();
    seqno.checked_add(Uint128::one());

    REQUEST_SEQNO_KEY.save(deps.storage, &seqno).unwrap();

    let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());

    let request = ReqState {
        reqtype: ReqType::TRANSFER,
        from: info.sender,
        to: Some(to),
        amount: amount,
        memo: Some(memo)
    };
    req_key.save(deps.storage, &request).unwrap();
    //TODO add event
    Ok(Response::default())
}

fn try_submit_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {

    let mut amount = Uint128::zero();

    for coin in &info.funds {
        amount += coin.amount
    }

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be transfered"));
    }

    let mut seqno = REQUEST_SEQNO_KEY.load(deps.storage).unwrap();
    seqno.checked_add(Uint128::one());

    REQUEST_SEQNO_KEY.save(deps.storage, &seqno).unwrap();

    let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());

    let request = ReqState {
        reqtype: ReqType::WITHDRAW,
        from: info.sender,
        to: None,
        amount: amount,
        memo: None
    };
    req_key.save(deps.storage, &request).unwrap();
    Ok(Response::default())
}

fn try_commit_response(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cipher: Binary,
) -> StdResult<Response> {

    let mut seqno = REQUEST_SEQNO_KEY.load(deps.storage).unwrap();
    seqno.checked_add(Uint128::one());

    REQUEST_SEQNO_KEY.save(deps.storage, &seqno).unwrap();

    let amount = Uint128::one(); //TODO from cipher
    let req_key = PREFIX_REQUESTS_KEY.add_suffix(&seqno.to_be_bytes());

    let request = ReqState {
        reqtype: ReqType::WITHDRAW,
        from: info.sender,
        to: None,
        amount: amount,
        memo: None
    };
    req_key.save(deps.storage, &request).unwrap();
    Ok(Response::default())
}

fn try_write_checkpoint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cipher: Binary,
) -> StdResult<Response> {
    let key = AEAD_KEY.load(deps.storage).unwrap();
    let checkpoint_vec = decrypt(cipher.as_slice(), &key).unwrap();
    let checkpoint_bin = to_binary(&checkpoint_vec).unwrap();
    let mut new_checkpoint: CheckPoint = from_binary(&checkpoint_bin).unwrap();

    let mut seqno = CHECKPOINT_SEQNO_KEY.load(deps.storage).unwrap();

    if seqno > new_checkpoint.seqno {
        return Err(StdError::generic_err("New Checkpoint Seq no too low"));
    }
    CHECKPOINT_SEQNO_KEY.save(deps.storage, &new_checkpoint.seqno).unwrap();

    let checkpoint_len_128: u128 = new_checkpoint.checkpoint.len().try_into().unwrap();
    let checkpoint_len = Uint128::from(checkpoint_len_128);
    CHECKPOINT_LEN_KEY.save(deps.storage, &checkpoint_len).unwrap();

    for i in 0..new_checkpoint.checkpoint.len() {
        let i_u128: u128 = i.try_into().unwrap();
        let num = Uint128::from(i_u128);
        let checkpt_key = PREFIX_CHECKPOINT_KEY.add_suffix(&num.to_be_bytes());
        let checkpt = new_checkpoint.checkpoint.get(i).unwrap();
        checkpt_key.save(deps.storage, &checkpt).unwrap();
    }

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
        } => get_checkpoint(deps, env)
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
    let mut seqno = CHECKPOINT_SEQNO_KEY.load(deps.storage).unwrap();
    let mut checkpoint_list = Vec::new();
    let checkpoint_len = CHECKPOINT_LEN_KEY.load(deps.storage).unwrap();
    for i in 0..checkpoint_len.u128() {
        let num = Uint128::from(i);
        let checkpt_key = PREFIX_CHECKPOINT_KEY.add_suffix(&num.to_be_bytes());
        let checkpt = checkpt_key.load(deps.storage).unwrap();
        checkpoint_list.push(checkpt);
    }

    let mut message = CheckPoint {
        checkpoint: checkpoint_list,
        seqno: seqno
    };
    let plaintext = to_binary(&message).unwrap();
    let entropy = env.block.random.unwrap();
    let key = AEAD_KEY.load(deps.storage).unwrap();

    let mut hasher = Sha256::default();
    hasher.update(entropy.clone().as_slice());
    let finalized_hash = hasher.finalize();
    let prng_seed = finalized_hash.as_slice();
    let mut rng = ContractPrng::new(&prng_seed, entropy.as_slice());
    let rnd_bytes = rng.rand_bytes();
    let nonce : [u8;12] = rnd_bytes[0..12].try_into().unwrap();

    let cipher = encrypt_with_nonce(&plaintext, &key, &nonce).unwrap();

    let resp_as_b64 = to_binary(&cipher).unwrap();

    Ok(resp_as_b64)
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
    use crate::msg::{ExecuteMsg, GetStateAnswer, InstantiateMsg, IterateHashAnswer, QueryMsg};

    #[test]
    fn test_get_state() {
        let mocked_env = mock_env();
        let mut mocked_deps = mock_dependencies();
        let mocked_info = mock_info("owner", &[]);

        let resp = instantiate(mocked_deps.as_mut(), mocked_env, mocked_info, InstantiateMsg {}).unwrap();

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

        let resp = instantiate(mocked_deps.as_mut(), mocked_env, mocked_info, InstantiateMsg {}).unwrap();

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


    #[test]
    fn test_checkpoint() {
        let mock_env = mock_env();
        let mut mock_deps = mock_dependencies();
        let mock_info = mock_info("owner", &[]);

        let resp = instantiate(mock_deps.as_mut(), mock_env, mock_info, InstantiateMsg{}); 

        let query_resp = get_checkpoint(mocked_deps.as_ref(), mocked_env).unwrap();

        println!("{:?}", query_resp.unwrap())
    }

}
