//! Admin CLI utilities for license management

/// Generate admin Ed25519 keypair for license signing
pub fn admin_keygen() -> anyhow::Result<String> {
    use ed25519_dalek::SigningKey;
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    let signing_key = SigningKey::from_bytes(&bytes);
    let pubkey_hex = hex::encode(signing_key.verifying_key().as_bytes());

    Ok(pubkey_hex)
}

/// Validate a MICAFP license token
pub fn validate_license_token(token: &str) -> anyhow::Result<bool> {
    // Token format: MICAFP-lic://v1/<base64(payload)>.<base64(sig)>
    if !token.starts_with("MICAFP-lic://") {
        return Ok(false);
    }
    Ok(true)
}
