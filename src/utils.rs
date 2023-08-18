use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
};
use generic_array::GenericArray;
use secret_toolkit_crypto::ContractPrng;
use crate::state::{SymmetricKey};
use cosmwasm_std::{StdError, Env, Uint128, Storage, Binary, to_binary, from_binary};
use sha2::{Sha256, Digest};
use cosmwasm_schema::cw_serde;


/// The IV key byte size
pub const IV_SIZE: usize = 96/8;
/// Type alias for the IV byte array
type IV = [u8; IV_SIZE];

#[cw_serde]
pub struct CipherText {
    pub cipher: Binary,
    pub nonce: Binary,
}

pub fn encrypt_with_nonce(message: &[u8], key: &SymmetricKey, iv: &IV) -> Result<CipherText, StdError> {
    let symmetric_key: &Key<Aes256Gcm> = key.into();
    let nonce = GenericArray::from_slice(iv);
    let cipher = Aes256Gcm::new(&symmetric_key);
    let ciphertext = cipher.encrypt(&nonce, message).unwrap();
    let ciphertext_vec = Vec::from(ciphertext);
    Ok(CipherText {
        cipher: to_binary(&ciphertext_vec).unwrap(),
        nonce: to_binary(&iv).unwrap(),
    })
}

pub fn decrypt_with_nonce(cipheriv: &[u8], key: &SymmetricKey, iv: &IV) -> Result<Vec<u8>, StdError> {
    let symmetric_key: &Key<Aes256Gcm> = key.into();
    let nonce = GenericArray::from_slice(iv);
    let cipher = Aes256Gcm::new(&symmetric_key);
    let plaintext = cipher.decrypt(&nonce, cipheriv).unwrap();
    let plaintext_vec = Vec::from(plaintext);
    Ok(plaintext_vec)
}
pub fn get_prng(seqno: Uint128, env: Env) -> ContractPrng {
    let entropy = env.block.random.unwrap();
    let seed: [u8;16] = seqno.to_be_bytes();
    let rng = ContractPrng::new(&seed, entropy.as_slice());
    return rng;
}

pub fn get_nonce(prng: &mut ContractPrng) -> IV {
    let rnd_bytes = prng.rand_bytes();
    let nonce : [u8;IV_SIZE] = rnd_bytes[0..IV_SIZE].try_into().unwrap();
    return nonce;
}

pub fn get_key(env: Env) -> SymmetricKey {
    let mut rng = ContractPrng::from_env(&env);
    let symmetric_key: SymmetricKey = rng.rand_bytes();
    return symmetric_key;
}

pub fn bool_to_uint128(b: bool) -> Uint128 {
    let c: u128 = b.try_into().unwrap();
    Uint128::from(c)
}

