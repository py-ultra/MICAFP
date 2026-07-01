# GitHub Codespaces — راهنمای کامل نصب خودکار

## آیا وابستگی‌ها بصورت خودکار نصب می‌شوند؟

**بله — کاملاً خودکار.** هنگامی که Codespace را باز می‌کنید، سه مرحله به ترتیب
اجرا می‌شوند:

| مرحله | فایل | زمان | محتوا |
|-------|------|------|-------|
| ۱ | `on-create.sh` | یک‌بار (build) | همه ابزارها و dependencies سیستمی |
| ۲ | `post-create.sh` | یک‌بار (اولین اجرا) | وابستگی‌های پروژه |
| ۳ | `post-start.sh` | هر بار (start) | بررسی محیط |

---

## چه چیزی خودکار نصب می‌شود

### زبان‌ها و Runtime‌ها
- **Rust** stable + تمام targets (Android, iOS, Linux musl, Windows)
- **Go** 1.22
- **Zig** 0.13.0 (برای TUN module و OpenWrt cross-compile)
- **Flutter** stable (برای اپ Android/iOS)
- **Node.js** 20 + bun (برای workers و dashboard)
- **Python** 3.x + scapy, cryptography, aiohttp

### ابزارهای سیستمی
- `clang` + `llvm` + `lld` — برای کامپایل eBPF C programs
- `libbpf-dev` — برای لود eBPF programs در Rust
- `protobuf-compiler` — برای gRPC/proto
- `android-ndk r26d` — برای build اندروید
- `iproute2`, `iptables`, `tcpdump` — برای تست شبکه

### cargo tools
```
cross        — cross-compilation for Android/iOS/Windows
cargo-deny   — dependency audit
cargo-audit  — security vulnerability check
cargo-watch  — auto-rebuild on file change
cargo-expand — macro expansion for debugging
```

---

## نصب دستی (اگر Codespaces استفاده نمی‌کنید)

### Ubuntu/Debian (WSL2, VPS, Native)
```bash
git clone <repo>
cd MICAFP-UnifiedShield
bash .devcontainer/on-create.sh       # نصب system dependencies
bash .devcontainer/post-create.sh     # نصب project dependencies
```

### macOS
```bash
# Homebrew prerequisites
brew install rustup go zig flutter bun clang llvm protobuf

# Rust setup
rustup default stable
rustup target add aarch64-linux-android aarch64-apple-ios

# Project
bash .devcontainer/post-create.sh
```

### Windows (WSL2 — توصیه می‌شود)
```powershell
# در PowerShell (admin):
wsl --install -d Ubuntu-24.04

# در WSL2 terminal:
bash .devcontainer/on-create.sh
bash .devcontainer/post-create.sh
```

---

## دستورات سریع پس از نصب

```bash
make dev          # Build daemon (debug) — سریع‌ترین
make test         # تمام تست‌ها
make release      # Full release build همه پلتفرم‌ها
make android      # Android .so library
make ios          # iOS .xcframework
make flutter      # Flutter APK + IPA
make zig-tun      # Zig TUN module (همه architectures)
make ebpf         # کامپایل eBPF C programs
make check        # cargo clippy + cargo audit
make docker       # Build Docker image
```

---

## رفع مشکل

### خطا: `linker 'cc' not found`
```bash
sudo apt-get install build-essential
```

### خطا: `zig: command not found`
```bash
export PATH="/opt/zig:$PATH"
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc
```

### خطا: `libbpf not found` (eBPF)
```bash
sudo apt-get install libbpf-dev linux-headers-$(uname -r)
```

### خطا: `ANDROID_NDK_HOME not set`
```bash
export ANDROID_NDK_HOME=/opt/android-ndk
```

### Flutter pub get خطا می‌دهد
```bash
flutter doctor     # بررسی مشکلات Flutter
flutter pub cache clean
flutter pub get
```

---

## نکته مهم برای eBPF

eBPF programs نیاز به kernel 5.4+ دارند. در Codespaces معمولاً
kernel 5.15+ است و بدون مشکل کار می‌کند. روی macOS نیاز به
Lima VM یا Docker دارید.

---

*UnifiedShield v8.0 | GitHub Codespaces Auto-Setup*
