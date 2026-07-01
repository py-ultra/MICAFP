use anyhow::{Context, Result};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, KeyInit};
use x25519_dalek::{EphemeralSecret, PublicKey};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use rand::Rng;
use base64::Engine;

type HmacSha256 = Hmac<Sha256>;

pub const FRAME_DATA: u8 = 0x01;
pub const FRAME_KEEPALIVE: u8 = 0x02;
pub const FRAME_RENEGOTIATE: u8 = 0x03;
pub const FRAME_CLOSE: u8 = 0x04;
pub const FRAME_PADDING: u8 = 0x05;

pub struct CloudflareWorkerTransport {
    url: String,
    session_key: Option<[u8; 32]>,
    send_nonce: u64,
    recv_nonce: u64,
}

impl CloudflareWorkerTransport {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), session_key: None, send_nonce: 0, recv_nonce: 0 }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let ephemeral_secret = EphemeralSecret::random();
        let ephemeral_public = PublicKey::from(&ephemeral_secret);
        let client_pub_b64 = base64::engine::general_purpose::STANDARD.encode(ephemeral_public.as_bytes());
        let shared_secret = ephemeral_secret.diffie_hellman(&PublicKey::from([0u8; 32]));
        let session_key = Self::derive_session_key(shared_secret.as_bytes(), &ephemeral_public.as_bytes());
        self.session_key = Some(session_key);
        tracing::info!("Cloudflare Worker transport: session key derived (Note: blocked in Iran, use Chinese CDN instead)");
        Ok(())
    }

    fn derive_session_key(shared_secret: &[u8], client_pub: &[u8]) -> [u8; 32] {
        let mut mac = HmacSha256::new_from_slice(b"unifiedshield-session-v1").unwrap();
        mac.update(shared_secret);
        mac.update(client_pub);
        let result = mac.finalize().into_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    pub fn encrypt_frame(&mut self, frame_type: u8, payload: &[u8]) -> Result<Vec<u8>> {
        let key_bytes = self.session_key.ok_or_else(|| anyhow::anyhow!("No session key"))?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&self.send_nonce.to_le_bytes());
        let mut rng = rand::thread_rng();
        let mut random_nonce = [0u8; 12];
        rng.fill_bytes(&mut random_nonce);
        random_nonce[4..12].copy_from_slice(&self.send_nonce.to_le_bytes());
        let ciphertext = cipher.encrypt(Nonce::from_slice(&random_nonce), payload)
            .map_err(|e| anyhow::anyhow!("Frame encrypt failed: {}", e))?;
        self.send_nonce = self.send_nonce.wrapping_add(1);
        let mut frame = Vec::with_capacity(4 + 1 + 12 + ciphertext.len());
        let total_len = 1 + 12 + ciphertext.len();
        frame.extend_from_slice(&(total_len as u32).to_le_bytes());
        frame.push(frame_type);
        frame.extend_from_slice(&random_nonce);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    pub fn decrypt_frame(&mut self, data: &[u8]) -> Result<(u8, Vec<u8>)> {
        if data.len() < 4 { return Err(anyhow::anyhow!("Frame too short")); }
        let len = u32::from_le_bytes(data[0..4].try_into()?) as usize;
        if data.len() < 4 + len { return Err(anyhow::anyhow!("Frame truncated")); }
        let frame_type = data[4];
        let nonce = &data[5..17];
        let ciphertext = &data[17..4+len];
        let key_bytes = self.session_key.ok_or_else(|| anyhow::anyhow!("No session key"))?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| anyhow::anyhow!("Frame decrypt failed: {}", e))?;
        self.recv_nonce = self.recv_nonce.wrapping_add(1);
        Ok((frame_type, plaintext))
    }

    pub fn create_padding_frame(&mut self) -> Result<Vec<u8>> {
        let mut rng = rand::thread_rng();
        let padding_size = rng.gen_range(16..256);
        let padding = vec![0u8; padding_size];
        self.encrypt_frame(FRAME_PADDING, &padding)
    }

    pub fn padding_interval_secs() -> f64 {
        use rand_distr::{Exp, Distribution};
        let exp = Exp::new(1.0 / 8.0).unwrap();
        exp.sample(&mut rand::thread_rng()).max(1.0)
    }
}
