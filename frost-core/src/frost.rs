//! An implementation of FROST (Flexible Round-Optimized Schnorr Threshold)
//! signatures.
//!
//! If you are interested in deploying FROST, please do not hesitate to consult the FROST authors.
//!
//! This implementation currently only supports key generation using a central
//! dealer. In the future, we will add support for key generation via a DKG,
//! as specified in the FROST paper.
//!
//! Internally, keygen_with_dealer generates keys using Verifiable Secret
//! Sharing, where shares are generated using Shamir Secret Sharing.

use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::{self, Debug},
};

use hex::FromHex;

mod identifier;
pub mod keys;
pub mod round1;
pub mod round2;

use crate::{Ciphersuite, Error, Field, Group, Signature};

pub use self::identifier::Identifier;

/// The binding factor, also known as _rho_ (ρ)
///
/// Ensures each signature share is strongly bound to a signing set, specific set
/// of commitments, and a specific message.
///
/// <https://github.com/cfrg/draft-irtf-cfrg-frost/blob/master/draft-irtf-cfrg-frost.md>
#[derive(Clone, PartialEq)]
pub struct Rho<C: Ciphersuite>(<<C::Group as Group>::Field as Field>::Scalar);

impl<C> Rho<C>
where
    C: Ciphersuite,
{
    /// Deserializes [`Rho`] from bytes.
    pub fn from_bytes(
        bytes: <<C::Group as Group>::Field as Field>::Serialization,
    ) -> Result<Self, Error> {
        <<C::Group as Group>::Field as Field>::deserialize(&bytes).map(|scalar| Self(scalar))
    }

    /// Serializes [`Rho`] to bytes.
    pub fn to_bytes(&self) -> <<C::Group as Group>::Field as Field>::Serialization {
        <<C::Group as Group>::Field as Field>::serialize(&self.0)
    }
}

impl<C> Debug for Rho<C>
where
    C: Ciphersuite,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Rho")
            .field(&hex::encode(self.to_bytes()))
            .finish()
    }
}

impl<C> From<&SigningPackage<C>> for Rho<C>
where
    C: Ciphersuite,
{
    // [`compute_binding_factor`] in the spec
    //
    // [`compute_binding_factor`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-03.html#section-4.4
    fn from(signing_package: &SigningPackage<C>) -> Rho<C> {
        let preimage = signing_package.rho_preimage();

        let binding_factor = C::H1(&preimage[..]);

        Rho(binding_factor)
    }
}

impl<C> FromHex for Rho<C>
where
    C: Ciphersuite,
{
    type Error = &'static str;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        match FromHex::from_hex(hex) {
            Ok(bytes) => Self::from_bytes(bytes).map_err(|_| "malformed scalar encoding"),
            Err(_) => Err("invalid hex"),
        }
    }
}

// TODO: pub struct Lagrange<C: Ciphersuite>(Scalar);

/// Generates the lagrange coefficient for the i'th participant.
fn derive_lagrange_coeff<C: Ciphersuite>(
    signer_id: u16,
    signing_package: &SigningPackage<C>,
) -> Result<<<C::Group as Group>::Field as Field>::Scalar, &'static str> {
    // This should fail and panic if signer_id_scalar is 0 in the scalar field.
    let signer_id_scalar = Identifier::<C>::try_from(signer_id).unwrap();

    let zero = <<C::Group as Group>::Field as Field>::zero();

    // TODO: This is redundant
    if signer_id_scalar.0 == zero {
        return Err("Invalid parameters");
    }

    if signing_package
        .signing_commitments()
        .iter()
        .any(|commitment| {
            let commitment_id_scalar = Identifier::<C>::try_from(commitment.index).unwrap();

            *commitment_id_scalar == zero
        })
    {
        return Err("Invalid parameters");
    }

    let mut num = <<C::Group as Group>::Field as Field>::one();
    let mut den = <<C::Group as Group>::Field as Field>::one();

    // Ala the sorting of B, just always sort by index in ascending order
    //
    // https://github.com/cfrg/draft-irtf-cfrg-frost/blob/master/draft-irtf-cfrg-frost.md#encoding-operations-dep-encoding
    for commitment in signing_package.signing_commitments() {
        if commitment.index == signer_id {
            continue;
        }

        let commitment_id_scalar = Identifier::<C>::try_from(commitment.index).unwrap();

        num = num * *commitment_id_scalar;
        den = den * (*commitment_id_scalar - *signer_id_scalar);
    }

    if den == zero {
        return Err("Duplicate shares provided");
    }

    // TODO(dconnolly): return this error if the inversion result == zero
    let lagrange_coeff = num * <<C::Group as Group>::Field as Field>::invert(&den).unwrap();

    Ok(lagrange_coeff)
}

/// Generated by the coordinator of the signing operation and distributed to
/// each signing party
pub struct SigningPackage<C: Ciphersuite> {
    /// The set of commitments participants published in the first round of the
    /// protocol.
    signing_commitments: HashMap<u16, round1::SigningCommitments<C>>,
    /// Message which each participant will sign.
    ///
    /// Each signer should perform protocol-specific verification on the
    /// message.
    message: Vec<u8>,
}

impl<C> SigningPackage<C>
where
    C: Ciphersuite,
{
    /// Create a new `SigingPackage`
    ///
    /// The `signing_commitments` are sorted by participant `index`.
    pub fn new(
        mut signing_commitments: Vec<round1::SigningCommitments<C>>,
        message: Vec<u8>,
    ) -> SigningPackage<C> {
        signing_commitments.sort_by_key(|a| a.index);

        SigningPackage {
            signing_commitments: signing_commitments
                .into_iter()
                .map(|s| (s.index, s))
                .collect(),
            message,
        }
    }

    /// Get a signing commitment by its participant index.
    pub fn signing_commitment(&self, index: &u16) -> round1::SigningCommitments<C> {
        self.signing_commitments[index]
    }

    /// Get the signing commitments, sorted by the participant indices
    pub fn signing_commitments(&self) -> Vec<round1::SigningCommitments<C>> {
        let mut signing_commitments: Vec<round1::SigningCommitments<C>> =
            self.signing_commitments.values().cloned().collect();
        signing_commitments.sort_by_key(|a| a.index);
        signing_commitments
    }

    /// Get the message to be signed
    pub fn message(&self) -> &Vec<u8> {
        &self.message
    }

    /// Compute the preimage to H3 to compute rho
    // We separate this out into its own method so it can be tested
    pub fn rho_preimage(&self) -> Vec<u8> {
        let mut preimage = vec![];

        preimage
            .extend_from_slice(&round1::encode_group_commitments(self.signing_commitments())[..]);
        preimage.extend_from_slice(C::H3(self.message.as_slice()).as_ref());

        preimage
    }
}

/// The product of all signers' individual commitments, published as part of the
/// final signature.
#[derive(PartialEq)]
pub struct GroupCommitment<C: Ciphersuite>(pub(super) <C::Group as Group>::Element);

// impl<C> Debug for GroupCommitment<C> where C: Ciphersuite {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_tuple("GroupCommitment")
//             .field(&hex::encode(self.0.compress().to_bytes()))
//             .finish()
//     }
// }

impl<C> TryFrom<&SigningPackage<C>> for GroupCommitment<C>
where
    C: Ciphersuite,
{
    type Error = &'static str;

    /// Generates the group commitment which is published as part of the joint
    /// Schnorr signature.
    ///
    /// Implements [`compute_group_commitment`] from the spec.
    ///
    /// [`compute_group_commitment`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-03.html#section-4.4
    fn try_from(signing_package: &SigningPackage<C>) -> Result<GroupCommitment<C>, &'static str> {
        let rho: Rho<C> = signing_package.into();

        let identity = <C::Group as Group>::identity();

        let mut accumulator = <C::Group as Group>::identity();

        // Ala the sorting of B, just always sort by index in ascending order
        //
        // https://github.com/cfrg/draft-irtf-cfrg-frost/blob/master/draft-irtf-cfrg-frost.md#encoding-operations-dep-encoding
        for commitment in signing_package.signing_commitments() {
            // The following check prevents a party from accidentally revealing their share.
            // Note that the '&&' operator would be sufficient.
            if identity == commitment.binding.0 || identity == commitment.hiding.0 {
                return Err("Commitment equals the identity.");
            }

            accumulator = accumulator + (commitment.hiding.0 + (commitment.binding.0 * rho.0))
        }

        Ok(GroupCommitment(accumulator))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Aggregation
////////////////////////////////////////////////////////////////////////////////

/// Verifies each participant's signature share, and if all are valid,
/// aggregates the shares into a signature to publish.
///
/// Resulting signature is compatible with verification of a plain SpendAuth
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
pub fn aggregate<C>(
    signing_package: &SigningPackage<C>,
    signature_shares: &[round2::SignatureShare<C>],
    pubkeys: &keys::PublicKeyPackage<C>,
) -> Result<Signature<C>, &'static str>
where
    C: Ciphersuite,
{
    // Encodes the signing commitment list produced in round one as part of generating [`Rho`], the
    // binding factor.
    let rho: Rho<C> = signing_package.into();

    // Compute the group commitment from signing commitments produced in round one.
    let group_commitment = GroupCommitment::<C>::try_from(signing_package)?;

    // Compute the per-message challenge.
    let challenge = crate::challenge::<C>(
        &group_commitment.0,
        &pubkeys.group_public.element,
        signing_package.message().as_slice(),
    );

    // Verify the signature shares.
    for signature_share in signature_shares {
        // Look up the public key for this signer, where `signer_pubkey` = _G.ScalarBaseMult(s[i])_,
        // and where s[i] is a secret share of the constant term of _f_, the secret polynomial.
        let signer_pubkey = pubkeys.signer_pubkeys.get(&signature_share.index).unwrap();

        // Compute Lagrange coefficient.
        let lambda_i = derive_lagrange_coeff(signature_share.index, signing_package)?;

        // Compute the commitment share.
        let R_share = signing_package
            .signing_commitment(&signature_share.index)
            .to_group_commitment_share(&rho);

        // Compute relation values to verify this signature share.
        signature_share.verify(&R_share, signer_pubkey, lambda_i, &challenge)?;
    }

    // The aggregation of the signature shares by summing them up, resulting in
    // a plain Schnorr signature.
    //
    // Implements [`frost_aggregate`] from the spec.
    //
    // [`frost_aggregate`]: https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-03.html#section-5.3-4
    let mut z = <<C::Group as Group>::Field as Field>::zero();

    for signature_share in signature_shares {
        z = z + signature_share.signature.z_share;
    }

    Ok(Signature {
        R: group_commitment.0,
        z,
    })
}
