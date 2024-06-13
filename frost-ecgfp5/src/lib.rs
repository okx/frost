#![allow(non_snake_case)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc = document_features::document_features!()]

use std::collections::BTreeMap;

use frost_core::{self as frost};
use frost_rerandomized::RandomizedCiphersuite;
use plonky2::{
    hash::poseidon::PoseidonHash,
    plonk::config::{GenericHashOut, Hasher},
};
use plonky2_ecgfp5::{
    curve::{curve::Point, scalar_field::Scalar},
    gadgets::poseidon::{hash_to_scalar, u8_to_goldilocks},
};
use plonky2_field::types::{Field as Plonky2Field, Sample};
use rand_core::{CryptoRng, RngCore};

#[cfg(test)]
mod tests;

// Re-exports in our public API
pub use frost_core::{serde, Ciphersuite, Field, FieldError, Group, GroupError};
pub use rand_core;

/// An error.
pub type Error = frost_core::Error<EcGFp5Poseidon256>;

/// An implementation of the FROST(ecGFp5, Poseidon-256) ciphersuite scalar field.
#[derive(Clone, Copy)]
pub struct EcGFp5ScalarField;

impl Field for EcGFp5ScalarField {
    type Scalar = Scalar;

    type Serialization = [u8; 40];

    fn zero() -> Self::Scalar {
        Scalar::ZERO
    }

    fn one() -> Self::Scalar {
        Scalar::ONE
    }

    fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
        scalar.try_inverse().ok_or(FieldError::InvalidZeroScalar)
    }

    fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self::Scalar {
        Scalar::sample(rng)
    }

    /// Little endian
    fn serialize(scalar: &Self::Scalar) -> Self::Serialization {
        scalar.encode()
    }

    /// Little endian
    fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
        Scalar::from_canonical_bytes(*buf).ok_or(FieldError::MalformedScalar)
    }

    fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
        scalar.encode()
    }
}

/// An implementation of the FROST(ecGFp5, Poseidon-256) ciphersuite group.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcGFp5Group;

impl Group for EcGFp5Group {
    type Field = EcGFp5ScalarField;

    type Element = Point;

    type Serialization = [u8; 40];

    fn cofactor() -> <Self::Field as Field>::Scalar {
        Scalar::ONE
    }

    fn identity() -> Self::Element {
        Point::NEUTRAL
    }

    fn generator() -> Self::Element {
        Point::GENERATOR
    }

    /// validate the element is not the group identity
    fn serialize(element: &Self::Element) -> Self::Serialization {
        element.to_le_bytes()
    }

    fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
        match Point::from_le_bytes(*buf) {
            Some(point) => {
                if point.is_neutral() {
                    Err(GroupError::InvalidIdentityElement)
                } else {
                    Ok(point)
                }
            }
            _ => Err(GroupError::MalformedElement),
        }
    }
}

fn hash_to_array(inputs: &[&[u8]]) -> [u8; 32] {
    let f_inputs = u8_to_goldilocks(inputs.concat().as_ref());
    PoseidonHash::hash_pad(&f_inputs)
        .to_bytes()
        .try_into()
        .unwrap()
}

/// Context string from the ciphersuite in the [spec].
///
/// [spec]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-14.html#section-6.5-1
const CONTEXT_STRING: &str = "FROST-ECGFP5-POSEIDON256-v1";

/// An implementation of the FROST(ecGFp5, Poseidon-256) ciphersuite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EcGFp5Poseidon256;

impl Ciphersuite for EcGFp5Poseidon256 {
    const ID: &'static str = CONTEXT_STRING;

    type Group = EcGFp5Group;

    type HashOutput = [u8; 32];

    type SignatureSerialization = [u8; 80];

    /// H1 for FROST(ecGFp5, Poseidon-256)
    fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
        hash_to_scalar((CONTEXT_STRING.to_owned() + "rho").as_bytes(), m)
    }

    /// H2 for FROST(ecGFp5, Poseidon-256)
    fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
        hash_to_scalar((CONTEXT_STRING.to_owned() + "chal").as_bytes(), m)
    }

    /// H3 for FROST(ecGFp5, Poseidon-256)
    fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
        hash_to_scalar((CONTEXT_STRING.to_owned() + "nonce").as_bytes(), m)
    }

    /// H4 for FROST(ecGFp5, Poseidon-256)
    fn H4(m: &[u8]) -> Self::HashOutput {
        hash_to_array(&[CONTEXT_STRING.as_bytes(), b"msg", m])
    }

    /// H5 for FROST(ecGFp5, Poseidon-256)
    fn H5(m: &[u8]) -> Self::HashOutput {
        hash_to_array(&[CONTEXT_STRING.as_bytes(), b"com", m])
    }

    /// HDKG for FROST(ecGFp5, Poseidon-256)
    fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
        Some(hash_to_scalar(
            (CONTEXT_STRING.to_owned() + "dkg").as_bytes(),
            m,
        ))
    }

    /// HID for FROST(ecGFp5, Poseidon-256)
    fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
        Some(hash_to_scalar(
            (CONTEXT_STRING.to_owned() + "id").as_bytes(),
            m,
        ))
    }
}

impl RandomizedCiphersuite for EcGFp5Poseidon256 {
    fn hash_randomizer(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
        Some(hash_to_scalar(
            (CONTEXT_STRING.to_owned() + "randomizer").as_bytes(),
            m,
        ))
    }
}

type S = EcGFp5Poseidon256;

/// A FROST(ecGFp5, Poseidon-256) participant identifier.
pub type Identifier = frost::Identifier<S>;

/// FROST(ecGFp5, Poseidon-256) keys, key generation, key shares.
pub mod keys {
    use super::*;
    use std::collections::BTreeMap;

    /// The identifier list to use when generating key shares.
    pub type IdentifierList<'a> = frost::keys::IdentifierList<'a, S>;

    /// Allows all participants' keys to be generated using a central, trusted
    /// dealer.
    pub fn generate_with_dealer<RNG: RngCore + CryptoRng>(
        max_signers: u16,
        min_signers: u16,
        identifiers: IdentifierList,
        mut rng: RNG,
    ) -> Result<(BTreeMap<Identifier, SecretShare>, PublicKeyPackage), Error> {
        frost::keys::generate_with_dealer(max_signers, min_signers, identifiers, &mut rng)
    }

    /// Splits an existing key into FROST shares.
    ///
    /// This is identical to [`generate_with_dealer`] but receives an existing key
    /// instead of generating a fresh one. This is useful in scenarios where
    /// the key needs to be generated externally or must be derived from e.g. a
    /// seed phrase.
    pub fn split<R: RngCore + CryptoRng>(
        secret: &SigningKey,
        max_signers: u16,
        min_signers: u16,
        identifiers: IdentifierList,
        rng: &mut R,
    ) -> Result<(BTreeMap<Identifier, SecretShare>, PublicKeyPackage), Error> {
        frost::keys::split(secret, max_signers, min_signers, identifiers, rng)
    }

    /// Recompute the secret from t-of-n secret shares using Lagrange interpolation.
    ///
    /// This can be used if for some reason the original key must be restored; e.g.
    /// if threshold signing is not required anymore.
    ///
    /// This is NOT required to sign with FROST; the whole point of FROST is being
    /// able to generate signatures only using the shares, without having to
    /// reconstruct the original key.
    ///
    /// The caller is responsible for providing at least `min_signers` shares;
    /// if less than that is provided, a different key will be returned.
    pub fn reconstruct(secret_shares: &[KeyPackage]) -> Result<SigningKey, Error> {
        frost::keys::reconstruct(secret_shares)
    }

    /// Secret and public key material generated by a dealer performing
    /// [`generate_with_dealer`].
    ///
    /// # Security
    ///
    /// To derive a FROST(ecGFp5, Poseidon-256) keypair, the receiver of the [`SecretShare`] *must* call
    /// .into(), which under the hood also performs validation.
    pub type SecretShare = frost::keys::SecretShare<S>;

    /// A secret scalar value representing a signer's share of the group secret.
    pub type SigningShare = frost::keys::SigningShare<S>;

    /// A public group element that represents a single signer's public verification share.
    pub type VerifyingShare = frost::keys::VerifyingShare<S>;

    /// A FROST(ecGFp5, Poseidon-256) keypair, which can be generated either by a trusted dealer or using
    /// a DKG.
    ///
    /// When using a central dealer, [`SecretShare`]s are distributed to
    /// participants, who then perform verification, before deriving
    /// [`KeyPackage`]s, which they store to later use during signing.
    pub type KeyPackage = frost::keys::KeyPackage<S>;

    /// Public data that contains all the signers' public keys as well as the
    /// group public key.
    ///
    /// Used for verification purposes before publishing a signature.
    pub type PublicKeyPackage = frost::keys::PublicKeyPackage<S>;

    /// Contains the commitments to the coefficients for our secret polynomial _f_,
    /// used to generate participants' key shares.
    ///
    /// [`VerifiableSecretSharingCommitment`] contains a set of commitments to the coefficients (which
    /// themselves are scalars) for a secret polynomial f, where f is used to
    /// generate each ith participant's key share f(i). Participants use this set of
    /// commitments to perform verifiable secret sharing.
    ///
    /// Note that participants MUST be assured that they have the *same*
    /// [`VerifiableSecretSharingCommitment`], either by performing pairwise comparison, or by using
    /// some agreed-upon public location for publication, where each participant can
    /// ensure that they received the correct (and same) value.
    pub type VerifiableSecretSharingCommitment = frost::keys::VerifiableSecretSharingCommitment<S>;

    pub mod dkg;
    pub mod repairable;
}

/// FROST(ecGFp5, Poseidon-256) Round 1 functionality and types.
pub mod round1 {
    use crate::keys::SigningShare;

    use super::*;

    /// Comprised of FROST(ecGFp5, Poseidon-256) hiding and binding nonces.
    ///
    /// Note that [`SigningNonces`] must be used *only once* for a signing
    /// operation; re-using nonces will result in leakage of a signer's long-lived
    /// signing key.
    pub type SigningNonces = frost::round1::SigningNonces<S>;

    /// Published by each participant in the first round of the signing protocol.
    ///
    /// This step can be batched if desired by the implementation. Each
    /// SigningCommitment can be used for exactly *one* signature.
    pub type SigningCommitments = frost::round1::SigningCommitments<S>;

    /// A commitment to a signing nonce share.
    pub type NonceCommitment = frost::round1::NonceCommitment<S>;

    /// Performed once by each participant selected for the signing operation.
    ///
    /// Generates the signing nonces and commitments to be used in the signing
    /// operation.
    pub fn commit<RNG>(secret: &SigningShare, rng: &mut RNG) -> (SigningNonces, SigningCommitments)
    where
        RNG: CryptoRng + RngCore,
    {
        frost::round1::commit::<S, RNG>(secret, rng)
    }
}

/// Generated by the coordinator of the signing operation and distributed to
/// each signing party.
pub type SigningPackage = frost::SigningPackage<S>;

/// FROST(ecGFp5, Poseidon-256) Round 2 functionality and types, for signature share generation.
pub mod round2 {
    use super::*;

    /// A FROST(ecGFp5, Poseidon-256) participant's signature share, which the Coordinator will aggregate with all other signer's
    /// shares into the joint signature.
    pub type SignatureShare = frost::round2::SignatureShare<S>;

    /// Performed once by each participant selected for the signing operation.
    ///
    /// Receives the message to be signed and a set of signing commitments and a set
    /// of randomizing commitments to be used in that signing operation, including
    /// that for this participant.
    ///
    /// Assumes the participant has already determined which nonce corresponds with
    /// the commitment that was assigned by the coordinator in the SigningPackage.
    pub fn sign(
        signing_package: &SigningPackage,
        signer_nonces: &round1::SigningNonces,
        key_package: &keys::KeyPackage,
    ) -> Result<SignatureShare, Error> {
        frost::round2::sign(signing_package, signer_nonces, key_package)
    }
}

/// A Schnorr signature on FROST(ecGFp5, Poseidon-256).
pub type Signature = frost_core::Signature<S>;

/// Verifies each FROST(ecGFp5, Poseidon-256) participant's signature share, and if all are valid,
/// aggregates the shares into a signature to publish.
///
/// Resulting signature is compatible with verification of a plain Schnorr
/// signature.
///
/// This operation is performed by a coordinator that can communicate with all
/// the signing participants before publishing the final signature. The
/// coordinator can be one of the participants or a semi-trusted third party
/// (who is trusted to not perform denial of service attacks, but does not learn
/// any secret information). Note that because the coordinator is trusted to
/// report misbehaving parties in order to avoid publishing an invalid
/// signature, if the coordinator themselves is a signer and misbehaves, they
/// can avoid that step. However, at worst, this results in a denial of
/// service attack due to publishing an invalid signature.
pub fn aggregate(
    signing_package: &SigningPackage,
    signature_shares: &BTreeMap<Identifier, round2::SignatureShare>,
    pubkeys: &keys::PublicKeyPackage,
) -> Result<Signature, Error> {
    frost::aggregate(signing_package, signature_shares, pubkeys)
}

/// A signing key for a Schnorr signature on FROST(ecGFp5, Poseidon-256).
pub type SigningKey = frost_core::SigningKey<S>;

/// A valid verifying key for Schnorr signatures on FROST(ecGFp5, Poseidon-256).
pub type VerifyingKey = frost_core::VerifyingKey<S>;
