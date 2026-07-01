# MICAFP-UnifiedShield-vip-ultra-Quantum-ultra-Quantum v9.0

**Complete merge of 16 source projects. Zero features removed.**

---

## Source Projects Merged (All 16)

| Project | Key Contribution |
|---------|-----------------|
| MICAFP-UnifiedShield (×13 versions) | Rust daemon, 22 protocols, AI/ML, PQC, mesh, CDN workers |
| **SlipNet** | DNSTT · NoizDNS · VayDNS · Slipstream QUIC · SSH-over-TLS/WS · NaiveProxy · DNS Scanner |
| **orbot-android** | Tor VPN (no root) · Snowflake · obfs4 · Meek-Azure · Tor control port |
| **MoaV** | Reality/VLESS · AmneziaWG · Hysteria2 · XHTTP · Shadowsocks-2022 · Docker server configs |

---

## Languages

Rust · Go · Kotlin · Swift · Flutter/Dart · C · WebAssembly · Zig

## Platforms

Android · iOS · Linux · Windows · OpenWrt · Chrome Extension · Firefox Extension

## Key Properties

- **No VPS required** — uses CDN workers, DNS tunneling, Tor, and Snowflake over public infrastructure
- **No root required** — Android VpnService API only
- **Zero compile errors** — all CI gates enforced
- **Fully automated CI/CD** — GitHub Actions builds and releases all platforms on tag push

---

## Quick Start

```bash
# Build the daemon (Linux)
cd daemon && cargo build --release --features full

# Run DNS scanner to find compatible resolvers
cd go-bridge/slipnet-dns && go run . --scan

# Build Android APK
cd flutter && flutter build apk --split-per-abi --release

# Build browser extensions
cd extensions/chrome && npm ci && npm run build
cd extensions/firefox && npm ci && npm run build
```

## CI/CD

Push a tag to trigger the full automated build and release:
```bash
git tag v9.0.0 && git push --tags
```

This triggers 13 CI jobs: lint, daemon tests, Go bridge tests, Linux (x86_64 + arm64),
Windows, Android APK, iOS, OpenWrt, Chrome extension, Firefox extension, WASM, DNS Scanner, and GitHub Release.

---

See [MERGE-MANIFEST.md](MERGE-MANIFEST.md) for full architecture documentation and [ENGINEERING-PROMPT.md](ENGINEERING-PROMPT.md) for the complete engineering spec.
