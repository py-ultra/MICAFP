# UnifiedShield — Next-Generation VPN Architecture Guide

## Version 7.0 | Quantum-Resistant | AI-DPI-Adversarial | NAIN-Resilient

---

## چه چیزی در ایران واقعاً جواب می‌دهد؟

### پروتکل‌های رتبه‌بندی‌شده (اردیبهشت ۱۴۰۴)

| رتبه | پروتکل | ISPها | دلیل موفقیت |
|------|--------|-------|-------------|
| ۱ | VLESS Reality (xtls-rprx-vision) | همه | TLS هرگز terminate نمی‌شود؛ DPI سرور واقعی می‌بیند |
| ۲ | ShadowTLS v3 | همه | handshake واقعی با Apple/Microsoft؛ active probing را شکست می‌دهد |
| ۳ | AmneziaWG | MCI, Shatel, ParsOnline, TCI | WireGuard با junk packets؛ signature کاملاً متفاوت |
| ۴ | NaiveProxy | Asiatech, Afranet, Rightel, Mobinnet | Chromium HTTP/2 stack؛ مثل Chrome به نظر می‌رسد |
| ۵ | Hysteria2 | Rightel, Asiatech, Afranet | QUIC-based؛ فقط ISPهایی که QUIC را مسدود نکرده‌اند |
| ۶ | VLESS-WS-TLS via ArvanCloud | همه | CDN fronting از طریق ArvanCloud داخلی |

### پروتکل‌های ناکارآمد در ایران ۱۴۰۴

- **WireGuard plain** — بلافاصله شناسایی می‌شود (signature ثابت byte 0-3)
- **OpenVPN UDP** — همه ISPها مسدود می‌کنند
- **Shadowsocks plain** — entropy analysis شناسایی می‌کند
- **Cloudflare CDN** — IP rangeهای Cloudflare کاملاً بلاک هستند

---

## حالت NAIN — وقتی اینترنت بین‌الملل قطع می‌شود

NAIN (National Information Network / شبکه ملی اطلاعات) یعنی تمام مسیرهای BGP بین‌المللی از TCI (مخابرات) withdrawal می‌شوند.

### چه چیزی در NAIN کار می‌کند؟

**ArvanCloud CDN** — تنها راه‌حل مطمئن. ArvanCloud در whitelist دولتی است. سرور VPN شما باید پشت ArvanCloud CDN قرار بگیرد (WebSocket + TLS). ترافیک داخل ایران به PoP ایرانی ArvanCloud می‌رود؛ backbone ArvanCloud transit بین‌المللی دارد که از withdrawal BGP جدا است.

**Yggdrasil overlay** — شبکه mesh با peers داخلی. نیازی به اینترنت مرکزی ندارد.

**SMS Bootstrap** — ارسال کانفیگ tunnel از طریق SMS برای زمانی که اینترنت کاملاً قطع است.

**BLE/WiFi-Aware Mesh** — اتصال محلی بدون اینترنت در شعاع ۱۰۰ متر.

---

## زبان‌ها و تکنولوژی‌هایی که اکثر پروژه‌ها ندارند

### ۱. Zig — برای کامپوننت‌های Kernel-level

Zig در این پروژه برای کراس‌کامپایل OpenWrt (MIPS/ARM) استفاده می‌شود. بدون toolchain خارجی؛ built-in cross-compilation.

```zig
// Zig build for MIPS OpenWrt — no OpenWrt SDK needed
zig build -Dtarget=mipsel-linux-musl -Doptimize=ReleaseSmall
```

**چرا ارزش دارد:** هیچ VPN متن‌بازی از Zig برای TUN device یا packet processor استفاده نمی‌کند. سریع‌تر از C، بدون undefined behavior، binary بسیار کوچک برای روترهای ۴-۱۶MB.

### ۲. eBPF — پردازش packet در kernel (صفر syscall)

```
nfqueue (userspace):  ~200-400 µs latency, high CPU
eBPF tc hook:         ~2-5 µs, near-zero CPU
eBPF XDP hook:        ~500 ns, minimal CPU
```

eBPF در این پروژه برای TLS fragmentation، TTL trick، و DISORDER mode در kernel اجرا می‌کند. GoodbyeDPI/Zapret این کار را در userspace انجام می‌دهند؛ ما در kernel.

### ۳. io_uring — پردازش async packet بدون syscall

```
epoll:    ~200k syscalls/sec برای 100k pps
io_uring: ~0 syscalls/sec در SQPOLL mode
```

برای VPN با ۲ میلیون packet/sec این یعنی ۱۶ core CPU صرف syscall نمی‌شود.

### ۴. WebAssembly (WASM) برای obfuscation logic

Obfuscation logic در WASM کامپایل می‌شود. هر بار که WASM اجرا می‌شود، binary signature متفاوت است. DPI-های signature-based نمی‌توانند الگو یاد بگیرند.

### ۵. Post-Quantum Cryptography (PQC) — NIST FIPS 203

```
Session key = HKDF-SHA512(
    X25519_shared_secret || ML-KEM-768_shared_secret
)
```

هیچ VPN تجاری بزرگی هنوز PQC ارائه نمی‌دهد. این یک مزیت رقابتی واقعی است.

### ۶. libp2p — P2P discovery بدون سرور مرکزی

اگر سرور بلاک شود، clientها از طریق libp2p DHT peers جدید پیدا می‌کنند. هیچ single point of failure وجود ندارد.

---

## معماری ماژول‌های جدید v7.0

```
daemon/src/
├── ebpf/                    ← NEW: kernel-level DPI bypass (eBPF tc/XDP)
│   ├── mod.rs               ← Manager, capability detection
│   ├── tls_splitter.rs      ← TLS fragmentation in kernel
│   ├── ttl_trick.rs         ← TTL manipulation at tc hook
│   └── packet_reorder.rs    ← DISORDER mode in kernel
│
├── io_uring/                ← NEW: zero-syscall packet processing
│   ├── mod.rs               ← Processor, mode selection
│   ├── tun_reader.rs        ← io_uring-based TUN device reads
│   └── zero_copy.rs         ← Registered buffer zero-copy I/O
│
├── obfuscation/
│   ├── shadow_tls/          ← NEW: ShadowTLS v3 full implementation
│   │   ├── mod.rs           ← Config, authenticator, SNI list
│   │   ├── client.rs        ← Client-side handshake + auth tag
│   │   └── hmac_auth.rs     ← HMAC-SHA256 authentication
│   │
│   ├── ai_dpi_adversarial.rs ← NEW: defeats FAVA v4 ML classifier
│   ├── amneziawg_advanced.rs ← NEW: per-ISP AmneziaWG config generator
│   ├── reality_advanced.rs   ← NEW: automated Reality dest discovery
│   └── pqc_hybrid.rs         ← NEW: ML-KEM-768 + X25519 hybrid
│
├── covert/
│   └── doh_over_arvancloud.rs ← NEW: NAIN-safe DNS with poison detection
│
└── national_intranet/
    └── arvancloud_relay.rs    ← NEW: ArvanCloud NAIN-safe transport
```

---

## راهنمای انتخاب پروتکل به تفکیک ISP

### همراه اول (MCI) — FAVA v3.2
**اول:** VLESS Reality با fingerprint Chrome و destination speedtest.net  
**دوم:** AmneziaWG با Jc=4 Jmin=40 Jmax=80  
**سوم:** ShadowTLS v3 با SNI captive.apple.com  
QUIC مسدود است؛ از Hysteria2 استفاده نکنید.

### ایرانسل — FAVA v4.0 + ML
**اول:** VLESS Reality با fingerprint **randomized** (هفته‌ای rotate)  
**دوم:** ShadowTLS v3 با SNI www.google.com + server-side auth  
**سوم:** NaiveProxy (HTTP/2 masquerade)  
FAVA v4 ML یاد می‌گیرد؛ fingerprint و destination را هفتگی تغییر دهید.

### پارس‌آنلاین — FAVA v4.1 + ML + Active Probing
**اول:** VLESS Reality با fingerprint randomized و short_id=16 bytes  
**دوم:** AmneziaWG با حداکثر obfuscation (Jc=8 Jmin=60)  
**سوم:** ShadowTLS v3 با strict_mode=true  
IP rotate هفتگی الزامی است. سخت‌ترین ISP ایران.

### شاتل — FAVA v3.5
**اول:** VLESS Reality با fingerprint Firefox  
**دوم:** AmneziaWG با Jc=4  
**سوم:** ShadowTLS v3 با SNI www.microsoft.com

### رایتل — FAVA v2.1
**اول:** هر پروتکلی کار می‌کند. Reality برای سرعت.  
**دوم:** Hysteria2 — QUIC مسدود نیست روی رایتل  
**سوم:** DefyxVPN  
ساده‌ترین ISP برای bypass.

### مخابرات — FAVA v2.5 + NAIN capability
**اول:** VLESS Reality (عادی)  
**دوم:** در NAIN mode: VLESS-WS-TLS via ArvanCloud (تنها گزینه)  
**سوم:** AmneziaWG با peers داخلی  
آماده‌سازی برای NAIN الزامی است.

---

## DNS در ایران — راه‌حل کامل

| روش | عادی | NAIN | توضیح |
|-----|------|------|-------|
| UDP 53 system | مسموم | مسموم | هرگز استفاده نکنید |
| Google 8.8.8.8 | بلاک | بلاک | IP بلاک است |
| Cloudflare 1.1.1.1 | بلاک | بلاک | IP بلاک است |
| DoH ArvanCloud | ✓ | ✓ | **اول انتخاب** |
| DoH Begzar.ir | ✓ | ✓ | **دوم انتخاب** |
| DoT ArvanCloud | ✓ | ✓ | پورت ۸۵۳ |
| DoH Electrodns | ✓ | احتمالی | سوم انتخاب |

**IP‌های مسموم شناخته‌شده (باید رد شوند):**
- 10.10.34.35 — poison کلاسیک TCI
- 185.51.200.2 — poison FAVA v3 (TCI backbone)
- 5.160.208.63 — poison ایرانسل

---

*UnifiedShield v7.0 — Architecture Document*  
*آخرین بروزرسانی: اردیبهشت ۱۴۰۴*
