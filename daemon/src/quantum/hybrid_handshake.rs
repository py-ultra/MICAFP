// ─────────────────────────────────────────────────────────────────────────────
// Quantum-Safe Hybrid Handshake
//
// Combines ML-KEM-1024 (FIPS 203) with X25519 Diffie-Hellman.
// Security: secure if EITHER classical OR post-quantum scheme is unbroken.
//
// Handshake flow:
//   1. Initiator generates:  (ek_kem, dk_kem) = KEM.KeyGen()
//                            (sk_dh, pk_dh) = X25519.KeyGen()
//   2. Initiator → Responder: ek_kem ‖ pk_dh
//   3. Responder encapsulates: (K_kem, ct_kem) = KEM.Encaps(ek_kem)
//                              K_dh = X25519(responder_sk, pk_dh)
//   4. Responder → Initiator: ct_kem
//   5. Both derive:           K = HKDF(K_kem ‖ K_dh, "MICAFP-v7-hybrid")
// ─────────────────────────────────────────────────────────────────────────────

use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::error::ShieldError;

/// Output of a completed hybrid handshake — shared secret ready for use.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct HybridSharedSecret(pub [u8; 32]);

/// Hybrid ML-KEM-1024 + X25519 handshake engine.
pub struct HybridHandshake {
    /// Whether this instance is the handshake initiator.
    pub is_initiator: bool,
}

impl HybridHandshake {
    pub fn new_initiator() -> Self { Self { is_initiator: true } }
    pub fn new_responder() -> Self { Self { is_initiator: false } }

    /// Derive the hybrid shared secret from the KEM and DH sub-secrets.
    /// Uses HKDF-SHA256 with domain separation label "MICAFP-v7-hybrid".
    pub fn derive_shared_secret(
        kem_secret: &[u8],
        dh_secret: &[u8],
    ) -> Result<HybridSharedSecret, ShieldError> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let mut ikm = Vec::with_capacity(kem_secret.len() + dh_secret.len());
        ikm.extend_from_slice(kem_secret);
        ikm.extend_from_slice(dh_secret);

        let hk = Hkdf::<Sha256>::new(None, &ikm);
        let mut okm = [0u8; 32];
        hk.expand(b"MICAFP-v7-hybrid", &mut okm)
            .map_err(|_| ShieldError::Crypto("HKDF expand failed".into()))?;

        Ok(HybridSharedSecret(okm))
    }
}
