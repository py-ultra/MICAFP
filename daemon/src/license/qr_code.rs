//! QR Code Generator for MICAFP License Tokens
//!
//! Encodes MICAFP-lic:// URIs as QR codes for admin distribution.
//! The QR code can be shared as:
//!   - PNG image file
//!   - SVG vector image
//!   - Terminal ASCII art (for CLI tools)
//!   - Base64-encoded PNG (for embedding in HTML/email)
//!
//! ## How to use as admin
//!
//! ```bash
//! # Generate and publish token + QR code
//! micafp-admin publish --expires 30d --output token.png
//!
//! # QR code contains: MICAFP-lic://v1/<payload>.<sig>
//! # User scans QR → app imports token directly (no server, no link)
//! ```
//!
//! ## QR Code Version / Data Capacity
//!
//! A MICAFP-lic:// token is approximately 300-500 bytes base64.
//! QR version 15-20 (alphanumeric mode) comfortably encodes this.
//! Error correction level M recommended (15% redundancy, smaller size).

use tracing::info;

/// Output format for QR code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrFormat {
    /// PNG binary (for files and display).
    Png,
    /// SVG text (for web/email embedding).
    Svg,
    /// ASCII art for terminal display.
    Terminal,
    /// Base64-encoded PNG for direct embedding.
    Base64Png,
}

/// Result of QR code generation.
#[derive(Debug)]
pub struct QrOutput {
    pub format: QrFormat,
    /// For Png: raw PNG bytes. For Svg/Terminal/Base64: UTF-8 string bytes.
    pub data: Vec<u8>,
    /// Size in pixels (for raster formats).
    pub size_px: Option<u32>,
    /// The original token URI encoded in the QR.
    pub token_uri: String,
}

impl QrOutput {
    pub fn as_string(&self) -> Option<String> {
        match self.format {
            QrFormat::Png => None,
            _ => String::from_utf8(self.data.clone()).ok(),
        }
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::write(path, &self.data)
    }
}

/// Generate a QR code for a MICAFP license token URI.
///
/// Production: uses the `qrcode` crate:
///   let code = QrCode::with_error_correction_level(token_uri, EcLevel::M)?;
///   let image = code.render::<Luma<u8>>().build();
///   image.save(path)?;
pub fn generate_qr(token_uri: &str, format: QrFormat, scale: u32) -> Result<QrOutput, QrError> {
    info!("Generating QR code: format={:?} scale={} uri_len={}", format, scale, token_uri.len());

    // Validate that this is a proper MICAFP token URI
    if !token_uri.starts_with("MICAFP-lic://") {
        return Err(QrError::InvalidToken("URI must start with MICAFP-lic://".into()));
    }

    match format {
        QrFormat::Terminal => {
            // ASCII art QR — works in any terminal
            let ascii = generate_ascii_qr(token_uri);
            Ok(QrOutput {
                format,
                data: ascii.into_bytes(),
                size_px: None,
                token_uri: token_uri.to_string(),
            })
        }
        QrFormat::Svg => {
            let svg = generate_svg_qr(token_uri, scale);
            Ok(QrOutput {
                format,
                data: svg.into_bytes(),
                size_px: Some(scale * 33),
                token_uri: token_uri.to_string(),
            })
        }
        QrFormat::Png | QrFormat::Base64Png => {
            // Production: use qrcode + image crates
            // Structural: return a placeholder
            let placeholder = b"PNG_PLACEHOLDER".to_vec();
            let data = if format == QrFormat::Base64Png {
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                STANDARD.encode(&placeholder).into_bytes()
            } else {
                placeholder
            };
            Ok(QrOutput {
                format,
                data,
                size_px: Some(scale * 33),
                token_uri: token_uri.to_string(),
            })
        }
    }
}

/// Generate a simple ASCII art QR representation (terminal display).
fn generate_ascii_qr(data: &str) -> String {
    // Production: use qrcode crate's render::<char>() method
    // Structural: generate a border with the data hint
    let len = data.len().min(40);
    let preview = &data[..len];
    format!(
        "┌{}┐\n│ MICAFP-QR ({} bytes) │\n│ {} │\n└{}┘\n[Scan with MICAFP app]",
        "─".repeat(preview.len() + 4),
        data.len(),
        preview,
        "─".repeat(preview.len() + 4)
    )
}

/// Generate a minimal SVG QR code placeholder.
fn generate_svg_qr(data: &str, scale: u32) -> String {
    let size = scale * 33;
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{s}" height="{s}"
           viewBox="0 0 {s} {s}">
          <rect width="{s}" height="{s}" fill="white"/>
          <text x="10" y="20" font-size="12" fill="black">MICAFP-QR</text>
          <text x="10" y="40" font-size="8" fill="grey">{preview}...</text>
          <!-- Production: qrcode crate SVG output goes here -->
        </svg>"#,
        s = size,
        preview = &data[..data.len().min(30)]
    )
}

#[derive(Debug, thiserror::Error)]
pub enum QrError {
    #[error("Invalid token URI: {0}")]
    InvalidToken(String),
    #[error("QR generation failed: {0}")]
    GenerationFailed(String),
}
