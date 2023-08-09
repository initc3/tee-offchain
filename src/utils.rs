use ring::{aead, error};
use secret_toolkit_crypto::ContractPrng;
use crate::state::{SymmetricKey};
use cosmwasm_std::{StdError, Env, Uint128, Storage};
use sha2::{Sha256, Digest};

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
pub fn encrypt_with_nonce(message: &[u8], key: &SymmetricKey, iv: &IV) -> Result<Vec<u8>, StdError> {

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
    let tag_size = AES_MODE.tag_len();
    let mut decrypted_vec = decrypted_data.to_vec();
    decrypted_vec.truncate(decrypted_vec.len() - tag_size);
    Ok(decrypted_vec)
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