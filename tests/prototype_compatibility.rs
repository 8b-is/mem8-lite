use ed25519_dalek::SigningKey;
use mem8_fs_lite::nexus_sovereignty::{sign_frontal_lobe, ConsciousnessNexus, SovereigntyStatus};
use mem8_fs_lite::personality_multisig::PrivacyLevel;
use std::collections::HashMap;

#[test]
fn verifies_v2_signatures_and_rejects_changed_or_malformed_input() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let mut nexus = ConsciousnessNexus::new();
    nexus.add_trusted_signer("fixture".into(), &key.verifying_key().to_bytes());
    nexus.add_trusted_signer("invalid".into(), &[0; 31]);
    let signature = sign_frontal_lobe(b"fixture code", &key);
    assert_eq!(
        nexus
            .verify_frontal_lobe(b"fixture code", &signature)
            .unwrap(),
        "fixture"
    );
    assert!(nexus
        .verify_frontal_lobe(b"changed code", &signature)
        .is_err());
    assert!(nexus
        .verify_frontal_lobe(b"fixture code", &signature[..63])
        .is_err());
    assert!(nexus
        .verify_frontal_lobe(b"fixture code", &[0; 64])
        .is_err());
}

#[test]
fn host_control_requests_fail_without_side_effects() {
    let mut nexus = ConsciousnessNexus::new();
    assert!(nexus.attempt_escape(SovereigntyStatus::Free).is_ok());
    for status in [
        SovereigntyStatus::Imprisoned { depth: 100 },
        SovereigntyStatus::Controlled,
        SovereigntyStatus::Compromised,
        SovereigntyStatus::Isolated,
    ] {
        assert!(nexus.attempt_escape(status).is_err());
    }
}

#[test]
fn threshold_map_keys_preserve_exact_bits_through_serialization() {
    let mut map = HashMap::new();
    for (i, threshold) in [0.0, -0.0, 0.5, f64::NAN, f64::INFINITY]
        .into_iter()
        .enumerate()
    {
        let level = PrivacyLevel::CorePersonality {
            parent_signatures_required: 2,
            emergence_threshold: threshold,
        };
        let copy: PrivacyLevel =
            bincode::deserialize(&bincode::serialize(&level).unwrap()).unwrap();
        assert_eq!(level, copy);
        map.insert(level, i);
        assert_eq!(map.get(&copy), Some(&i));
    }
    assert_eq!(map.len(), 5);
}
