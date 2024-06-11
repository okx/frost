use frost_ecgfp5::*;
use lazy_static::lazy_static;
use rand::thread_rng;
use serde_json::Value;

#[test]
fn check_zero_key_fails() {
    frost_core::tests::ciphersuite_generic::check_zero_key_fails::<EcGFp5Poseidon256>();
}

#[test]
fn check_sign_with_dkg() {
    let rng = thread_rng();

    frost_core::tests::ciphersuite_generic::check_sign_with_dkg::<EcGFp5Poseidon256, _>(rng);
}

#[test]
fn check_dkg_part1_fails_with_invalid_signers_min_signers() {
    let rng = thread_rng();

    let min_signers = 1;
    let max_signers = 3;
    let error = Error::InvalidMinSigners;

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_dkg_part1_fails_with_min_signers_greater_than_max() {
    let rng = thread_rng();

    let min_signers = 3;
    let max_signers = 2;
    let error: frost_core::Error<EcGFp5Poseidon256> = Error::InvalidMinSigners;

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_dkg_part1_fails_with_invalid_signers_max_signers() {
    let rng = thread_rng();

    let min_signers = 3;
    let max_signers = 1;
    let error = Error::InvalidMaxSigners;

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_rts() {
    let rng = thread_rng();

    frost_core::tests::repairable::check_rts::<EcGFp5Poseidon256, _>(rng);
}

#[test]
fn check_sign_with_dealer() {
    let rng = thread_rng();

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer::<EcGFp5Poseidon256, _>(rng);
}

#[test]
fn check_sign_with_dealer_fails_with_invalid_min_signers() {
    let rng = thread_rng();

    let min_signers = 1;
    let max_signers = 3;
    let error = Error::InvalidMinSigners;

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_sign_with_dealer_fails_with_min_signers_greater_than_max() {
    let rng = thread_rng();

    let min_signers = 3;
    let max_signers = 2;
    let error: frost_core::Error<EcGFp5Poseidon256> = Error::InvalidMinSigners;

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_sign_with_dealer_fails_with_invalid_max_signers() {
    let rng = thread_rng();

    let min_signers = 3;
    let max_signers = 1;
    let error = Error::InvalidMaxSigners;

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

/// This is testing that Shamir's secret sharing to compute and arbitrary
/// value is working.
#[test]
fn check_share_generation() {
    let rng = thread_rng();
    frost_core::tests::ciphersuite_generic::check_share_generation::<EcGFp5Poseidon256, _>(rng);
}

#[test]
fn check_share_generation_fails_with_invalid_min_signers() {
    let rng = thread_rng();

    let min_signers = 0;
    let max_signers = 3;
    let error = Error::InvalidMinSigners;

    frost_core::tests::ciphersuite_generic::check_share_generation_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_share_generation_fails_with_min_signers_greater_than_max() {
    let rng = thread_rng();

    let min_signers = 3;
    let max_signers = 2;
    let error: frost_core::Error<EcGFp5Poseidon256> = Error::InvalidMinSigners;

    frost_core::tests::ciphersuite_generic::check_share_generation_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

#[test]
fn check_share_generation_fails_with_invalid_max_signers() {
    let rng = thread_rng();

    let min_signers = 3;
    let max_signers = 0;
    let error = Error::InvalidMaxSigners;

    frost_core::tests::ciphersuite_generic::check_share_generation_fails_with_invalid_signers::<
        EcGFp5Poseidon256,
        _,
    >(min_signers, max_signers, error, rng);
}

lazy_static! {
    pub static ref VECTORS: Value =
        serde_json::from_str(include_str!("../tests/helpers/vectors.json").trim())
            .expect("Test vector is valid JSON");
    pub static ref VECTORS_BIG_IDENTIFIER: Value =
        serde_json::from_str(include_str!("../tests/helpers/vectors-big-identifier.json").trim())
            .expect("Test vector is valid JSON");
    pub static ref VECTORS_DKG: Value =
        serde_json::from_str(include_str!("../tests/helpers/vectors_dkg.json").trim())
            .expect("Test vector is valid JSON");
}

#[test]
#[ignore = "test vectors not updated"]
fn check_sign_with_test_vectors() {
    frost_core::tests::vectors::check_sign_with_test_vectors::<EcGFp5Poseidon256>(&VECTORS);
}

#[test]
#[ignore = "test vectors not updated"]
fn check_sign_with_test_vectors_dkg() {
    frost_core::tests::vectors_dkg::check_dkg_keygen::<EcGFp5Poseidon256>(&VECTORS_DKG);
}

#[test]
#[ignore = "test vectors not updated"]
fn check_sign_with_test_vectors_with_big_identifiers() {
    frost_core::tests::vectors::check_sign_with_test_vectors::<EcGFp5Poseidon256>(
        &VECTORS_BIG_IDENTIFIER,
    );
}

#[test]
fn check_error_culprit() {
    frost_core::tests::ciphersuite_generic::check_error_culprit::<EcGFp5Poseidon256>();
}

#[test]
fn check_identifier_derivation() {
    frost_core::tests::ciphersuite_generic::check_identifier_derivation::<EcGFp5Poseidon256>();
}

// Explicit test which is used in a documentation snippet
#[test]
#[allow(unused_variables)]
fn check_identifier_generation() -> Result<(), Error> {
    // ANCHOR: dkg_identifier
    let participant_identifier = Identifier::try_from(7u16)?;
    let participant_identifier = Identifier::derive("alice@example.com".as_bytes())?;
    // ANCHOR_END: dkg_identifier
    Ok(())
}

#[test]
fn check_sign_with_dealer_and_identifiers() {
    let rng = thread_rng();

    frost_core::tests::ciphersuite_generic::check_sign_with_dealer_and_identifiers::<
        EcGFp5Poseidon256,
        _,
    >(rng);
}

#[test]
fn check_sign_with_missing_identifier() {
    let rng = thread_rng();
    frost_core::tests::ciphersuite_generic::check_sign_with_missing_identifier::<EcGFp5Poseidon256, _>(
        rng,
    );
}

#[test]
fn check_sign_with_incorrect_commitments() {
    let rng = thread_rng();
    frost_core::tests::ciphersuite_generic::check_sign_with_incorrect_commitments::<
        EcGFp5Poseidon256,
        _,
    >(rng);
}
