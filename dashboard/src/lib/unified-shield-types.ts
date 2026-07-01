export type CoreStatus = 'connected' | 'disconnected' | 'connecting' | 'error' | 'standby';

export type ProtocolType =
  | 'amneziawg-1.5'
  | 'vless-reality-xtls'
  | 'vless-fragment'
  | 'vmess-ws-tls'
  | 'trojan-grpc'
  | 'hysteria2'
  | 'tuic-v5'
  | 'shadowtls-v3'
  | 'psiphon-ssh-obfs'
  | 'lantern-df-pt'
  | 'mahsang-obfs'
  | 'mvless'
  | 'defyxvpn-layers'
  | 'moav-tunnel'
  | 'wireguard-noise'
  | 'shadowsocks-shadowtls'
  | 'psiphon-cdn-front'
  | 'naiveproxy';

export type PlatformType = 'android' | 'windows' | 'linux' | 'ios' | 'openwrt' | 'macos';

export interface PlatformSupport {
  platform: PlatformType;
  nameFa: string;
  supported: boolean;
  tunnelType: string;
  tunnelTypeFa: string;
  minVersion: string;
  icon: string;
}

export interface HealthStatus {
  latency: number;
  packetLoss: number;
  blocked: boolean;
  dnsLeak: boolean;
  dpiExposure: number;
  uptime: number;
  bandwidth: { up: number; down: number };
}

export interface CoreAdapter {
  id: string;
  name: string;
  nameFa: string;
  version: string;
  latestVersion: string;
  status: CoreStatus;
  priority: number;
  health: HealthStatus;
  capabilities: ProtocolType[];
  lastChecked: number;
  blockEvents24h: number;
  color: string;
  icon: string;
  description: string;
  descriptionFa: string;
  githubUrl: string;
  githubApiUrl: string;
  assetFilter: string;
  role: string;
  roleFa: string;
  specialFeatures: string[];
  specialFeaturesFa: string[];
  platforms: PlatformType[];
  checksumSha256: string;
}

export interface AIOrchestratorState {
  activeCoreId: string;
  shadowConnections: string[];
  scoringMatrix: Record<string, number>;
  ucbScores: Record<string, { exploitation: number; exploration: number; total: number }>;
  predictionState: {
    imminentBlockRisk: number;
    predictedBlockCore: string | null;
    proactiveSwitchRecommended: boolean;
  };
  rlWeights: Record<string, number[]>;
  learningRate: number;
  totalSwitches: number;
  successfulSwitches: number;
  averageSwitchTime: number;
  detectedISP: string;
  detectedISPFa: string;
  ispRuleApplied: string;
}

export interface ISPRule {
  id: string;
  name: string;
  nameFa: string;
  preferredCores: string[];
  blockedProtocols: string[];
  blockedProtocolsFa: string[];
  bestObfuscation: string[];
  bestObfuscationFa: string[];
}

export interface TrafficRoutingState {
  mode: 'full-vpn' | 'split-tunnel' | 'selective';
  iranIpBypass: boolean;
  dnsMode: 'doh' | 'dot' | 'plain';
  dnsProviders: string[];
  activeDnsProvider: string;
  ipv6Enabled: boolean;
  p2pRouting: boolean;
  splitRules: SplitRule[];
}

export interface SplitRule {
  id: string;
  app: string;
  appFa: string;
  route: 'vpn' | 'direct';
  enabled: boolean;
}

export interface OTAUpdateState {
  lastCheck: number;
  nextCheck: number;
  updates: OTAUpdate[];
  autoUpdate: boolean;
  rollbackEnabled: boolean;
  sha256Verification: boolean;
  checkIntervalHours: number;
}

export interface OTAUpdate {
  id: string;
  type: 'core-binary' | 'block-db' | 'ai-weights' | 'node-list';
  target: string;
  version: string;
  currentVersion: string;
  size: number;
  deltaPatch: boolean;
  signature: string;
  sha256: string;
  status: 'available' | 'downloading' | 'installed' | 'failed';
  githubReleaseUrl: string;
}

export interface DPITestResult {
  coreId: string;
  coreName: string;
  coreNameFa: string;
  connected: boolean;
  latency: number;
  protocol: string;
  bypassLevel: 'full' | 'partial' | 'none';
  dpiSignature: string;
  dpiSignatureFa: string;
  timestamp: number;
}

export interface ConnectionStats {
  totalUptime: number;
  totalDataTransferred: { up: number; down: number };
  coresUsed: number;
  switchesPerformed: number;
  blockEventsAvoided: number;
  currentSpeed: { up: number; down: number };
  activePlatform: PlatformType;
}

export const PLATFORMS: PlatformSupport[] = [
  { platform: 'android', nameFa: 'اندروید', supported: true, tunnelType: 'VpnService', tunnelTypeFa: 'سرویس VPN (بدون روت)', minVersion: '5.0+', icon: '🤖' },
  { platform: 'windows', nameFa: 'ویندوز', supported: true, tunnelType: 'Wintun/TAP', tunnelTypeFa: 'درایور Wintun', minVersion: '7+', icon: '🪟' },
  { platform: 'linux', nameFa: 'لینوکس', supported: true, tunnelType: 'tun/tap', tunnelTypeFa: 'رابط tun/tap', minVersion: 'Kernel 4.x+', icon: '🐧' },
  { platform: 'ios', nameFa: 'آی‌اواس', supported: true, tunnelType: 'NEPacketTunnelProvider', tunnelTypeFa: 'اکستنشن شبکه (بدون جیلبریک)', minVersion: '15+', icon: '🍎' },
  { platform: 'openwrt', nameFa: 'اوپن‌دبلیو‌آر‌تی', supported: true, tunnelType: 'netifd/tun', tunnelTypeFa: 'تونل از طریق netifd', minVersion: '21.02+', icon: '📦' },
  { platform: 'macos', nameFa: 'مک‌اواس', supported: true, tunnelType: 'NEPacketTunnelProvider', tunnelTypeFa: 'اکستنشن شبکه', minVersion: '12+', icon: '💻' },
];

export const ISP_RULES: ISPRule[] = [
  {
    id: 'mci',
    name: 'MCI (Hamrahe Avval)',
    nameFa: 'همراه اول',
    preferredCores: ['mahsang', 'amneziavpn'],
    blockedProtocols: ['VMess', 'VLESS_plain'],
    blockedProtocolsFa: ['VMess', 'VLESS ساده'],
    bestObfuscation: ['fragment', 'warp_noise'],
    bestObfuscationFa: ['Fragment', 'نویز Warp'],
  },
  {
    id: 'irancell',
    name: 'Irancell (MTN)',
    nameFa: 'ایرانسل',
    preferredCores: ['hiddify', 'defyxvpn'],
    blockedProtocols: ['Shadowsocks_plain'],
    blockedProtocolsFa: ['شادوساکس ساده'],
    bestObfuscation: ['reality', 'hysteria2'],
    bestObfuscationFa: ['Reality', 'هیستریا۲'],
  },
  {
    id: 'shatel',
    name: 'Shatel',
    nameFa: 'شتل',
    preferredCores: ['amneziavpn', 'psiphon'],
    blockedProtocols: ['WireGuard_plain'],
    blockedProtocolsFa: ['وایرگارد ساده'],
    bestObfuscation: ['amneziawg_junk', 'tls_fragment'],
    bestObfuscationFa: ['آمنزیاوی‌جی جونک', 'Fragment TLS'],
  },
  {
    id: 'asiatech',
    name: 'Asiatech',
    nameFa: 'آسیاتک',
    preferredCores: ['mahsang', 'hiddify'],
    blockedProtocols: ['HTTP_plain', 'SOCKS5'],
    blockedProtocolsFa: ['HTTP ساده', 'SOCKS5'],
    bestObfuscation: ['fake_host', 'doh_fragment'],
    bestObfuscationFa: ['میزبان جعلی', 'Fragment DoH'],
  },
  {
    id: 'rightel',
    name: 'Rightel',
    nameFa: 'رایتل',
    preferredCores: ['defyxvpn', 'hiddify'],
    blockedProtocols: [],
    blockedProtocolsFa: [],
    bestObfuscation: ['reality_vless', 'hysteria2'],
    bestObfuscationFa: ['VLESS Reality', 'هیستریا۲'],
  },
];

export const CORE_DEFINITIONS: Omit<CoreAdapter, 'status' | 'health' | 'priority' | 'lastChecked' | 'blockEvents24h'>[] = [
  {
    id: 'hiddify',
    name: 'hiddify-core',
    nameFa: 'هیدیفای',
    version: 'v4.1.0',
    latestVersion: 'v4.1.0',
    capabilities: ['vless-reality-xtls', 'vmess-ws-tls', 'trojan-grpc', 'hysteria2', 'tuic-v5', 'shadowtls-v3', 'naiveproxy', 'shadowsocks-shadowtls'],
    color: '#10b981',
    icon: '🛡️',
    description: 'Primary orchestration core; handles all sing-box based protocols',
    descriptionFa: 'هسته هماهنگ‌سازی اصلی؛ مدیریت تمام پروتکل‌های مبتنی بر sing-box',
    githubUrl: 'https://github.com/hiddify/hiddify-core',
    githubApiUrl: 'https://api.github.com/repos/hiddify/hiddify-core/releases/latest',
    assetFilter: 'hiddify-core-android-arm64',
    role: 'Primary orchestration core',
    roleFa: 'هسته هماهنگ‌سازی اصلی',
    specialFeatures: ['Auto-protocol selection', 'Built-in sing-box', 'Free node support', 'Multi-protocol fallback'],
    specialFeaturesFa: ['انتخاب خودکار پروتکل', 'sing-box داخلی', 'پشتیبانی از نود رایگان', 'بکاپ چند پروتکلی'],
    platforms: ['android', 'windows', 'linux', 'ios', 'openwrt', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'xray-gfw',
    name: 'GFW-knocker/Xray-core',
    nameFa: 'ایکس‌ری GFW',
    version: 'v25.8.3-mahsa-r1',
    latestVersion: 'v25.8.3-mahsa-r1',
    capabilities: ['vless-reality-xtls', 'vless-fragment', 'vmess-ws-tls', 'trojan-grpc', 'wireguard-noise', 'mvless'],
    color: '#6366f1',
    icon: '⚡',
    description: 'Specialized Iran bypass; Fragment+DoH, WireGuard noise, FakeHost, UDP noise',
    descriptionFa: 'عبور تخصصی از فیلترینگ ایران؛ Fragment+DoH، نویز WireGuard، FakeHost، نویز UDP',
    githubUrl: 'https://github.com/GFW-knocker/Xray-core',
    githubApiUrl: 'https://api.github.com/repos/GFW-knocker/Xray-core/releases/latest',
    assetFilter: 'Xray-android-arm64-v8a',
    role: 'Specialized Iran bypass engine',
    roleFa: 'موتور تخصصی عبور از فیلترینگ ایران',
    specialFeatures: ['Custom WireGuard noise', 'TLS fragmentor', 'Fake host injection', 'QUIC manipulation', 'DoH fragment', 'MVLESS protocol'],
    specialFeaturesFa: ['نویز سفارشی WireGuard', 'Fragment TLS', 'تزریق میزبان جعلی', 'دستکاری QUIC', 'Fragment DoH', 'پروتکل MVLESS'],
    platforms: ['android', 'windows', 'linux', 'ios', 'openwrt', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'sing-box',
    name: 'sing-box',
    nameFa: 'سینگ‌باکس',
    version: 'v1.14.0-alpha.25',
    latestVersion: 'v1.14.0-alpha.25',
    capabilities: ['hysteria2', 'tuic-v5', 'shadowtls-v3', 'vless-reality-xtls', 'vmess-ws-tls', 'shadowsocks-shadowtls', 'naiveproxy'],
    color: '#f59e0b',
    icon: '📦',
    description: 'Universal proxy platform — bundled as hiddify-core dependency',
    descriptionFa: 'پلتفرم پروکسی جامع — به عنوان وابستگی هیدیفای گنجانده شده',
    githubUrl: 'https://github.com/SagerNet/sing-box',
    githubApiUrl: 'https://api.github.com/repos/SagerNet/sing-box/releases/latest',
    assetFilter: 'sing-box-.*-android-arm64',
    role: 'Protocol handler (embedded in hiddify)',
    roleFa: 'مدیریت پروتکل‌ها (داخلی هیدیفای)',
    specialFeatures: ['ShadowTLS v3', 'Hysteria2 UDP obfuscation', 'TUIC v5 QUIC multiplex', 'NaiveProxy support'],
    specialFeaturesFa: ['شدوتی‌ال‌اس نسخه ۳', 'پنهان‌سازی UDP هیستریا۲', 'مولتی‌پلکس QUIC نسخه ۵ TUIC', 'پشتیبانی NaiveProxy'],
    platforms: ['android', 'windows', 'linux', 'ios', 'openwrt', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'amneziavpn',
    name: 'AmneziaVPN (awg-go)',
    nameFa: 'آمنزیاوی‌پی‌ان',
    version: '4.8.15.4',
    latestVersion: '4.8.15.0',
    capabilities: ['amneziawg-1.5'],
    color: '#ec4899',
    icon: '🔐',
    description: 'WireGuard with obfuscation headers — most effective against DPI in Russia/Iran',
    descriptionFa: 'وایرگارد با هدرهای پنهان‌سازی — مؤثرترین در برابر DPI روسیه و ایران',
    githubUrl: 'https://github.com/amnezia-vpn/awg-go',
    githubApiUrl: 'https://api.github.com/repos/amnezia-vpn/awg-go/releases/latest',
    assetFilter: 'awg-go-android',
    role: 'AmneziaWG protocol handler',
    roleFa: 'مدیر پروتکل آمنزیاوی‌جی',
    specialFeatures: ['Junk packet injection', 'Transport header obfuscation', 'AmneziaWG 1.5 custom headers'],
    specialFeaturesFa: ['تزریق بسته‌های جونک', 'پنهان‌سازی هدر ترانسپورت', 'هدرهای سفارشی آمنزیاوی‌جی ۱.۵'],
    platforms: ['android', 'windows', 'linux', 'ios', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'defyxvpn',
    name: 'DefyxVPN',
    nameFa: 'دیفیکسوی‌پی‌ان',
    version: 'v5.2.8',
    latestVersion: 'v5.2.8',
    capabilities: ['defyxvpn-layers', 'vless-reality-xtls', 'amneziawg-1.5'],
    color: '#8b5cf6',
    icon: '🌀',
    description: 'High-speed bypass; P2P support; unlimited bandwidth',
    descriptionFa: 'عبور پرسرعت؛ پشتیبانی P2P؛ پهنای باند نامحدود',
    githubUrl: 'https://github.com/UnboundTechCo/defyxVPN',
    githubApiUrl: 'https://api.github.com/repos/UnboundTechCo/defyxVPN/releases/latest',
    assetFilter: 'defyx-android',
    role: 'High-speed bypass with P2P',
    roleFa: 'عبور پرسرعت با P2P',
    specialFeatures: ['VLESS Reality', 'AmneziaWG 1.5', 'Unlimited bandwidth', 'P2P/torrenting support'],
    specialFeaturesFa: ['VLESS Reality', 'آمنزیاوی‌جی ۱.۵', 'پهنای باند نامحدود', 'پشتیبانی P2P/تورنت'],
    platforms: ['android', 'windows', 'linux', 'ios', 'openwrt', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'moav',
    name: 'MoaV',
    nameFa: 'موآوی',
    version: 'v1.7.7',
    latestVersion: 'v1.7.7',
    capabilities: ['moav-tunnel'],
    color: '#14b8a6',
    icon: '🌊',
    description: 'Advanced tunnel protocol with adaptive obfuscation',
    descriptionFa: 'پروتکل تونل پیشرفته با پنهان‌سازی تطبیقی',
    githubUrl: 'https://github.com/GFW-knocker/MahsaNG',
    githubApiUrl: 'https://api.github.com/repos/GFW-knocker/MahsaNG/releases/latest',
    assetFilter: 'moav-android',
    role: 'Adaptive tunnel engine',
    roleFa: 'موتور تونل تطبیقی',
    specialFeatures: ['Adaptive obfuscation', 'Multi-path routing', 'Dynamic key rotation'],
    specialFeaturesFa: ['پنهان‌سازی تطبیقی', 'مسیریابی چندمسیره', 'چرخش کلید پویا'],
    platforms: ['android', 'windows', 'linux', 'ios', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'lantern',
    name: 'Lantern',
    nameFa: 'لنترن',
    version: 'v7.9.0',
    latestVersion: 'v7.9.0',
    capabilities: ['lantern-df-pt', 'psiphon-cdn-front'],
    color: '#f97316',
    icon: '🏮',
    description: 'Domain fronting with pluggable transports for censorship bypass',
    descriptionFa: 'فرانتینگ دامنه با حمل‌های قابل تعویض برای عبور از سانسور',
    githubUrl: 'https://github.com/getlantern/lantern',
    githubApiUrl: 'https://api.github.com/repos/getlantern/lantern/releases/latest',
    assetFilter: 'lantern-android',
    role: 'Domain fronting transport',
    roleFa: 'حمل فرانتینگ دامنه',
    specialFeatures: ['Domain fronting', 'Pluggable transports', 'Peer-to-peer fallback', 'CDN leveraging'],
    specialFeaturesFa: ['فرانتینگ دامنه', 'حمل‌های قابل تعویض', 'بکاپ همتا به همتا', 'استفاده از CDN'],
    platforms: ['android', 'windows', 'linux', 'ios', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'mahsang',
    name: 'MahsaNG core',
    nameFa: 'مهساان‌جی',
    version: 'v26.3.31-mahsa-r1',
    latestVersion: 'v26.3.31-mahsa-r1',
    capabilities: ['mahsang-obfs', 'vless-reality-xtls', 'vless-fragment', 'vmess-ws-tls', 'mvless', 'wireguard-noise'],
    color: '#ef4444',
    icon: '✊',
    description: 'Iranian-specific customizations; MVLESS protocol; rotating configs',
    descriptionFa: 'سفارشی‌سازی‌های ایران؛ پروتکل MVLESS؛ پیکربندی چرخشی',
    githubUrl: 'https://github.com/GFW-knocker/MahsaNG',
    githubApiUrl: 'https://api.github.com/repos/GFW-knocker/MahsaNG/releases/latest',
    assetFilter: 'libv2ray.aar',
    role: 'Iran-optimized bypass engine',
    roleFa: 'موتور عبور بهینه‌شده برای ایران',
    specialFeatures: ['YouTube Direct bypass', 'HTTPS/TLS DoH fragmentor', 'Warp noise', 'MVLESS protocol', 'Rotating configs'],
    specialFeaturesFa: ['عبور مستقیم یوتیوب', 'Fragment DoH HTTPS/TLS', 'نویز Warp', 'پروتکل MVLESS', 'پیکربندی چرخشی'],
    platforms: ['android', 'windows', 'linux', 'ios', 'macos'],
    checksumSha256: 'verified',
  },
  {
    id: 'psiphon',
    name: 'Psiphon Tunnel Core (GFW-knocker)',
    nameFa: 'سایفون',
    version: 'latest',
    latestVersion: 'latest',
    capabilities: ['psiphon-ssh-obfs', 'psiphon-cdn-front'],
    color: '#06b6d4',
    icon: '🔵',
    description: 'Last-resort fallback when all other cores are blocked — connects independently without VPS',
    descriptionFa: 'بکاپ آخرین مرحله وقتی همه هسته‌ها مسدودند — بدون نیاز به سرور متصل می‌شود',
    githubUrl: 'https://github.com/GFW-knocker/psiphon-tunnel-core',
    githubApiUrl: 'https://api.github.com/repos/GFW-knocker/psiphon-tunnel-core/releases/latest',
    assetFilter: 'psiphon-tunnel-core-android',
    role: 'Fallback layer (last resort)',
    roleFa: 'لایه بکاپ (آخرین مرحله)',
    specialFeatures: ['SSH transport + ObfuscatedSSH', 'CDN domain fronting', 'No VPS required', 'Independent connectivity'],
    specialFeaturesFa: ['حمل SSH + SSH مبهم', 'فرانتینگ دامنه CDN', 'بدون نیاز به سرور', 'اتصال مستقل'],
    platforms: ['android', 'windows', 'linux', 'ios', 'openwrt', 'macos'],
    checksumSha256: 'verified',
  },
];

export const PROTOCOL_LABELS: Record<ProtocolType, { name: string; nameFa: string; color: string; antiDpiMethod: string; antiDpiMethodFa: string; priority: number }> = {
  'vless-reality-xtls': { name: 'VLESS + XTLS-Reality', nameFa: 'VLESS Reality + XTLS', color: '#6366f1', antiDpiMethod: 'Reality handshake mimicry', antiDpiMethodFa: 'تقلید دست‌دهی Reality', priority: 1 },
  'amneziawg-1.5': { name: 'AmneziaWG 1.5', nameFa: 'آمنزیاوی‌جی ۱.۵', color: '#ec4899', antiDpiMethod: 'Junk packet injection', antiDpiMethodFa: 'تزریق بسته‌های جونک', priority: 2 },
  'hysteria2': { name: 'Hysteria2', nameFa: 'هیستریا۲', color: '#8b5cf6', antiDpiMethod: 'UDP obfuscation', antiDpiMethodFa: 'پنهان‌سازی UDP', priority: 3 },
  'wireguard-noise': { name: 'WireGuard + Noise', nameFa: 'وایرگارد + نویز', color: '#14b8a6', antiDpiMethod: 'UDP noise packets', antiDpiMethodFa: 'بسته‌های نویز UDP', priority: 4 },
  'vless-fragment': { name: 'VLESS + Fragment', nameFa: 'VLESS + Fragment', color: '#6366f1', antiDpiMethod: 'TLS fragment split', antiDpiMethodFa: 'تقسیم Fragment TLS', priority: 5 },
  'shadowtls-v3': { name: 'ShadowTLS v3', nameFa: 'شدوتی‌ال‌اس نسخه ۳', color: '#06b6d4', antiDpiMethod: 'TLS handshake masking', antiDpiMethodFa: 'پوشش دست‌دهی TLS', priority: 6 },
  'tuic-v5': { name: 'TUIC v5', nameFa: 'TUIC نسخه ۵', color: '#14b8a6', antiDpiMethod: 'QUIC multiplexing', antiDpiMethodFa: 'مولتی‌پلکس QUIC', priority: 7 },
  'trojan-grpc': { name: 'Trojan + WS + TLS', nameFa: 'تروجان + WS + TLS', color: '#f59e0b', antiDpiMethod: 'CDN fronting', antiDpiMethodFa: 'فرانتینگ CDN', priority: 8 },
  'psiphon-cdn-front': { name: 'Psiphon CDN-Front', nameFa: 'سایفون فرانت CDN', color: '#06b6d4', antiDpiMethod: 'CDN domain fronting', antiDpiMethodFa: 'فرانتینگ دامنه CDN', priority: 9 },
  'shadowsocks-shadowtls': { name: 'SS + ShadowTLS', nameFa: 'شادوساکس + شدوتی‌ال‌اس', color: '#f59e0b', antiDpiMethod: 'Double obfuscation', antiDpiMethodFa: 'پنهان‌سازی دوگانه', priority: 10 },
  'vmess-ws-tls': { name: 'VMess + WS + TLS', nameFa: 'VMess + WS + TLS', color: '#10b981', antiDpiMethod: 'WebSocket tunneling', antiDpiMethodFa: 'تونل WebSocket', priority: 11 },
  'mvless': { name: 'MVLESS', nameFa: 'ام‌وی‌لس (MVLESS)', color: '#ef4444', antiDpiMethod: 'MahsaNG custom obfuscation', antiDpiMethodFa: 'پنهان‌سازی سفارشی مهساان‌جی', priority: 12 },
  'psiphon-ssh-obfs': { name: 'Psiphon SSH+Obfs', nameFa: 'سایفون SSH+Obfs', color: '#f97316', antiDpiMethod: 'Obfuscated SSH transport', antiDpiMethodFa: 'حمل SSH مبهم‌شده', priority: 13 },
  'lantern-df-pt': { name: 'Lantern DF+PT', nameFa: 'لنترن DF+PT', color: '#ef4444', antiDpiMethod: 'Domain fronting + transports', antiDpiMethodFa: 'فرانتینگ دامنه + حمل‌ها', priority: 14 },
  'mahsang-obfs': { name: 'MahsaNG Obfs', nameFa: 'مهساان‌جی Obfs', color: '#ef4444', antiDpiMethod: 'Iran-specific custom obfuscation', antiDpiMethodFa: 'پنهان‌سازی سفارشی ایران', priority: 15 },
  'defyxvpn-layers': { name: 'DefyxVPN Layers', nameFa: 'لایه‌های دیفیکس', color: '#8b5cf6', antiDpiMethod: 'Pluggable obfuscation layers', antiDpiMethodFa: 'لایه‌های پنهان‌سازی قابل تعویض', priority: 16 },
  'moav-tunnel': { name: 'MoaV Tunnel', nameFa: 'تونل موآوی', color: '#14b8a6', antiDpiMethod: 'Adaptive tunnel obfuscation', antiDpiMethodFa: 'پنهان‌سازی تونل تطبیقی', priority: 17 },
  'naiveproxy': { name: 'NaiveProxy', nameFa: 'نایوپروکسی', color: '#10b981', antiDpiMethod: 'Chrome network stack mimicry', antiDpiMethodFa: 'تقلید پشته شبکه کروم', priority: 18 },
};

export const IRAN_IP_RANGES = [
  '5.160.0.0/12', '31.56.0.0/14', '37.32.0.0/14', '46.32.0.0/12',
  '62.60.0.0/14', '77.36.0.0/14', '78.38.0.0/14', '80.191.0.0/14',
  '84.47.0.0/14', '85.9.0.0/14', '86.57.0.0/14', '91.92.0.0/14',
  '92.50.0.0/14', '93.110.0.0/14', '94.101.0.0/14', '95.80.0.0/14',
  '109.72.0.0/14', '151.232.0.0/14', '159.20.0.0/14', '164.215.0.0/14',
  '176.65.0.0/14', '178.22.0.0/14', '185.2.0.0/14', '188.121.0.0/14',
  '194.225.0.0/14', '213.176.0.0/14', '217.11.0.0/14', '217.146.0.0/14',
];

export const DNS_PROVIDERS = [
  { id: 'cloudflare', name: 'Cloudflare', url: 'https://1.1.1.1/dns-query', dotUrl: '1.1.1.1', nameFa: 'کلودفلر' },
  { id: 'google', name: 'Google', url: 'https://8.8.8.8/dns-query', dotUrl: '8.8.8.8', nameFa: 'گوگل' },
  { id: 'quad9', name: 'Quad9', url: 'https://9.9.9.9/dns-query', dotUrl: '9.9.9.9', nameFa: 'کواد۹' },
  { id: 'shecan', name: 'Shecan', url: 'https://178.22.122.100/dns-query', dotUrl: '178.22.122.100', nameFa: 'شکن' },
];

export const IRAN_DPI_SIGNATURES = [
  { signature: 'TLS-ClientHello-Reset', descriptionFa: 'بازنشانی ClientHello TLS — شایع‌ترین روش DPI ایران', hex: '16 03 01' },
  { signature: 'HTTP-403-Block', descriptionFa: 'صفحه ۴۰۳ ایرانی — مسدودسازی HTTP', hex: '48 54 54 50 2F 31 2E 31 20 34 30 33' },
  { signature: 'Null-Route', descriptionFa: 'مسیریابی صفر — قطعی بی‌صدا', hex: '00 00 00 00' },
  { signature: 'SNI-Filter', descriptionFa: 'فیلتر SNI — بررسی نام سرور در TLS', hex: 'SNI-Filter-Detected' },
  { signature: 'DNS-Poison', descriptionFa: 'مسمومیت DNS — پاسخ جعلی DNS', hex: 'DNS-Poison-Response' },
  { signature: 'Protocol-Detect', descriptionFa: 'تشخیص پروتکل — شناسایی الگوی پروتکل', hex: 'Protocol-Pattern-Match' },
];

// ──────────────────────────────────────────────
// Kill Switch Types
// ──────────────────────────────────────────────
export interface KillSwitchState {
  enabled: boolean;
  blockAllOnDisconnect: boolean;
  allowedApps: string[];
  networkLock: boolean;
}

// ──────────────────────────────────────────────
// Auto-Reconnect Types
// ──────────────────────────────────────────────
export interface AutoReconnectState {
  enabled: boolean;
  maxRetries: number;
  retryCount: number;
  retryInterval: number;
  exponentialBackoff: boolean;
  lastReconnectAttempt: number;
  reconnectStatus: 'idle' | 'reconnecting' | 'failed' | 'connected';
}

// ──────────────────────────────────────────────
// Connection Log Types
// ──────────────────────────────────────────────
export interface ConnectionLogEntry {
  id: string;
  timestamp: number;
  type: 'connect' | 'disconnect' | 'switch' | 'block' | 'reconnect' | 'dpi-detect' | 'update' | 'error';
  message: string;
  messageFa: string;
  coreId?: string;
  details?: Record<string, string>;
}

// ──────────────────────────────────────────────
// Threat Intelligence Types
// ──────────────────────────────────────────────
export interface ThreatEntry {
  id: string;
  type: string;
  typeFa: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  descriptionFa: string;
  detectedAt: number;
  mitigated: boolean;
  countermeasure: string;
  countermeasureFa: string;
}

export interface ThreatIntelState {
  activeThreats: ThreatEntry[];
  lastScan: number;
  threatLevel: 'low' | 'medium' | 'high' | 'critical';
  dpiPatternsUpdated: string;
  blockedDomainsCount: number;
  activeCountermeasures: string[];
}

// ──────────────────────────────────────────────
// Advanced Settings Types
// ──────────────────────────────────────────────
export interface AdvancedSettings {
  language: 'fa' | 'en';
  theme: 'dark' | 'light' | 'system';
  startOnBoot: boolean;
  autoConnectOnLaunch: boolean;
  notifications: boolean;
  stealthMode: boolean;
  debugMode: boolean;
  connectionTimeout: number;
  mtuSize: number;
}

// ──────────────────────────────────────────────
// Network Analyzer Types
// ──────────────────────────────────────────────
export type NetworkType = 'wifi' | 'mobile-data' | 'ethernet' | 'unknown';

export interface BandwidthHistoryEntry {
  timestamp: number;
  uploadMbps: number;
  downloadMbps: number;
  latencyMs: number;
  packetLoss: number;
}

export interface TrafficBreakdown {
  protocol: string;
  protocolFa: string;
  bytesUp: number;
  bytesDown: number;
  percentage: number;
  color: string;
}

export interface DataUsageEntry {
  date: string;
  uploadMb: number;
  downloadMb: number;
  totalMb: number;
}

export interface PacketStats {
  sent: number;
  received: number;
  retransmitted: number;
  lost: number;
  retransmitRate: number;
  lossRate: number;
}

export interface NetworkAnalyzerState {
  isMonitoring: boolean;
  currentUploadMbps: number;
  currentDownloadMbps: number;
  currentLatencyMs: number;
  currentPacketLoss: number;
  currentJitter: number;
  networkType: NetworkType;
  networkTypeFa: string;
  connectionQualityScore: number;
  connectionQualityLabel: string;
  connectionQualityLabelFa: string;
  stabilityIndex: number;
  stabilityLabel: string;
  stabilityLabelFa: string;
  bandwidthHistory: BandwidthHistoryEntry[];
  trafficBreakdown: TrafficBreakdown[];
  packetStats: PacketStats;
  dataUsageDaily: DataUsageEntry[];
  dataUsageWeekly: DataUsageEntry[];
  dataUsageMonthly: DataUsageEntry[];
  totalDataUsedMb: number;
  monitoringStartTime: number;
}

// ──────────────────────────────────────────────
// Geographic Router Types
// ──────────────────────────────────────────────
export interface ServerCountry {
  code: string;
  name: string;
  nameFa: string;
  servers: number;
  activeServers: number;
  avgLatencyMs: number;
  loadPercent: number;
  isHealthy: boolean;
  lastPingMs: number;
  lastChecked: number;
  bandwidthCapacity: number;
  currentLoad: number;
  supportsIranBypass: boolean;
  features: string[];
  featuresFa: string[];
}

export interface GeoLatencyMap {
  [countryCode: string]: {
    latencyMs: number;
    jitterMs: number;
    packetLoss: number;
    lastMeasured: number;
  };
}

export interface LoadBalancingState {
  strategy: 'round-robin' | 'least-connections' | 'lowest-latency' | 'weighted';
  strategyFa: string;
  enabled: boolean;
  currentDistribution: Record<string, number>;
}

export interface GeoRouterState {
  selectedCountry: string;
  selectedCountryFa: string;
  serverList: ServerCountry[];
  latencyMap: GeoLatencyMap;
  loadBalancing: LoadBalancingState;
  autoSelectEnabled: boolean;
  iranInternalBypass: boolean;
  healthCheckInterval: number;
  lastHealthCheck: number;
  recommendedCountry: string;
  recommendedCountryFa: string;
  recommendationReason: string;
  recommendationReasonFa: string;
}

// ──────────────────────────────────────────────
// Security Audit Types
// ──────────────────────────────────────────────
export interface SecurityRecommendation {
  id: string;
  category: string;
  categoryFa: string;
  title: string;
  titleFa: string;
  description: string;
  descriptionFa: string;
  severity: 'info' | 'warning' | 'critical';
  action: string;
  actionFa: string;
  implemented: boolean;
}

export interface DNSLeakResult {
  isLeaking: boolean;
  detectedServers: string[];
  expectedServer: string;
  leakCount: number;
  totalQueries: number;
  testDurationMs: number;
  details: string;
  detailsFa: string;
}

export interface WebRTCLeakResult {
  isLeaking: boolean;
  detectedIPs: string[];
  localIPs: string[];
  publicIPs: string[];
  details: string;
  detailsFa: string;
}

export interface IPv6LeakResult {
  isLeaking: boolean;
  ipv6Address: string | null;
  expectedIPv6: string | null;
  details: string;
  detailsFa: string;
}

export interface EncryptionAssessment {
  protocol: string;
  protocolFa: string;
  keyExchange: string;
  keyExchangeFa: string;
  cipher: string;
  cipherFa: string;
  strength: 'weak' | 'moderate' | 'strong' | 'excellent';
  strengthFa: string;
  score: number;
}

export interface SecurityAuditState {
  isRunning: boolean;
  lastAuditTime: number;
  privacyScore: number;
  privacyScoreLabel: string;
  privacyScoreLabelFa: string;
  dnsLeak: DNSLeakResult;
  webrtcLeak: WebRTCLeakResult;
  ipv6Leak: IPv6LeakResult;
  killSwitchVerified: boolean;
  killSwitchDetails: string;
  killSwitchDetailsFa: string;
  encryptionAssessment: EncryptionAssessment;
  recommendations: SecurityRecommendation[];
  realTimeMonitoring: boolean;
  lastRealTimeCheck: number;
  overallSecurityStatus: 'secure' | 'warning' | 'vulnerable' | 'critical';
  overallSecurityStatusFa: string;
}

export const COUNTRY_SERVERS = [
  { code: 'DE', name: 'Germany', nameFa: 'آلمان', servers: 24 },
  { code: 'NL', name: 'Netherlands', nameFa: 'هلند', servers: 18 },
  { code: 'FI', name: 'Finland', nameFa: 'فنلاند', servers: 8 },
  { code: 'SE', name: 'Sweden', nameFa: 'سوئد', servers: 6 },
  { code: 'FR', name: 'France', nameFa: 'فرانسه', servers: 12 },
  { code: 'US', name: 'USA', nameFa: 'آمریکا', servers: 30 },
  { code: 'CA', name: 'Canada', nameFa: 'کانادا', servers: 10 },
  { code: 'GB', name: 'UK', nameFa: 'انگلستان', servers: 14 },
  { code: 'JP', name: 'Japan', nameFa: 'ژاپن', servers: 8 },
  { code: 'KR', name: 'South Korea', nameFa: 'کره جنوبی', servers: 6 },
  { code: 'SG', name: 'Singapore', nameFa: 'سنگاپور', servers: 10 },
  { code: 'AU', name: 'Australia', nameFa: 'استرالیا', servers: 5 },
  { code: 'BR', name: 'Brazil', nameFa: 'برزیل', servers: 4 },
  { code: 'IN', name: 'India', nameFa: 'هند', servers: 8 },
  { code: 'TR', name: 'Turkey', nameFa: 'ترکیه', servers: 16 },
  { code: 'AE', name: 'UAE', nameFa: 'امارات', servers: 6 },
  { code: 'CH', name: 'Switzerland', nameFa: 'سوئیس', servers: 8 },
  { code: 'NO', name: 'Norway', nameFa: 'نروژ', servers: 4 },
  { code: 'PL', name: 'Poland', nameFa: 'لهستان', servers: 6 },
  { code: 'ES', name: 'Spain', nameFa: 'اسپانیا', servers: 5 },
];
