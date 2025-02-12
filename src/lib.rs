#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod config;
mod deserializer;
pub mod validate;
mod vk;

use deserializer::{deserialize_proof_with_pubs, deserialize_vk};

use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::config::{GenericConfig, KeccakGoldilocksConfig, PoseidonGoldilocksConfig};
use snafu::Snafu;

pub use config::Plonky2Config;
pub use deserializer::{custom::ZKVerifyGateSerializer, DeserializeError};
pub use vk::Vk;

/// Verification error.
#[derive(Debug, Snafu)]
pub enum VerifyError {
    /// Invalid data.
    #[snafu(display("Invalid data for verification: [{}]", cause))]
    InvalidData {
        /// Internal error.
        #[snafu(source)]
        cause: DeserializeError,
    },
    /// Failure.
    #[snafu(display("Failed to verify"))]
    Failure,
}

impl From<DeserializeError> for VerifyError {
    fn from(value: DeserializeError) -> Self {
        VerifyError::InvalidData { cause: value }
    }
}

/// Verify the given `proof` and public inputs `pubs` using verification key `vk`.
pub fn verify_inner<F, C, const D: usize>(
    vk: &[u8],
    proof: &[u8],
    pubs: &[u8],
) -> Result<(), VerifyError>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    let vk = deserialize_vk::<F, C, D>(vk)?;
    let proof = deserialize_proof_with_pubs::<F, C, D>(proof, pubs, &vk.common)?;

    vk.verify(proof).map_err(|_| VerifyError::Failure)
}

/// Verification with preset Poseidon over Goldilocks config available in `plonky2`.
pub fn verify_default_poseidon(vk: &[u8], proof: &[u8], pubs: &[u8]) -> Result<(), VerifyError> {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    verify_inner::<F, C, D>(vk, proof, pubs)
}

/// Verification with preset Keccak over Goldilocks config available in `plonky2`.
pub fn verify_default_keccak(vk: &[u8], proof: &[u8], pubs: &[u8]) -> Result<(), VerifyError> {
    const D: usize = 2;
    type C = KeccakGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    verify_inner::<F, C, D>(vk, proof, pubs)
}

/// Verify `proof` with `pubs` depending on `vk` plonky2 configuration.
pub fn verify(vk: &Vk, proof: &[u8], pubs: &[u8]) -> Result<(), VerifyError> {
    match vk.config {
        Plonky2Config::Keccak => verify_default_keccak(&vk.bytes, proof, pubs),
        Plonky2Config::Poseidon => verify_default_poseidon(&vk.bytes, proof, pubs),
    }
}
