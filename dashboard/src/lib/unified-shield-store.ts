import { create } from 'zustand';
import {
  CoreAdapter,
  CoreStatus,
  HealthStatus,
  AIOrchestratorState,
  TrafficRoutingState,
  OTAUpdateState,
  DPITestResult,
  ConnectionStats,
  KillSwitchState,
  AutoReconnectState,
  ConnectionLogEntry,
  ThreatEntry,
  ThreatIntelState,
  AdvancedSettings,
  NetworkAnalyzerState,
  GeoRouterState,
  SecurityAuditState,
  CORE_DEFINITIONS,
  DNS_PROVIDERS,
  ISP_RULES,
  IRAN_DPI_SIGNATURES,
} from './unified-shield-types';
import {
  buildInitialNetworkAnalyzerState,
  updateNetworkMonitoring,
} from './network-analyzer';
import {
  buildInitialGeoRouterState,
  selectServerCountry as geoSelectServerCountry,
} from './geo-router';
import {
  buildInitialSecurityAuditState,
  runFullSecurityAudit,
} from './security-audit';

function generateHealth(coreId: string, isActive: boolean): HealthStatus {
  const baseLatency: Record<string, number> = {
    'hiddify': 85, 'xray-gfw': 62, 'sing-box': 73,
    'amneziavpn': 91, 'defyxvpn': 105, 'moav': 118,
    'lantern': 142, 'mahsang': 79, 'psiphon': 156,
  };
  const base = baseLatency[coreId] ?? 100;
  const jitter = Math.random() * 30 - 15;
  const latency = isActive ? Math.max(20, Math.round(base + jitter)) : 0;
  const packetLoss = isActive ? Math.random() * 3 : 0;
  const blocked = isActive ? Math.random() < 0.05 : false;
  const dnsLeak = isActive ? Math.random() < 0.02 : false;
  const dpiExposure = isActive ? Math.random() * 15 : 100;

  return {
    latency,
    packetLoss: Math.round(packetLoss * 100) / 100,
    blocked,
    dnsLeak,
    dpiExposure: Math.round(dpiExposure * 10) / 10,
    uptime: isActive ? Math.floor(Math.random() * 86400) : 0,
    bandwidth: isActive
      ? { up: Math.round(Math.random() * 50 + 10), down: Math.round(Math.random() * 200 + 50) }
      : { up: 0, down: 0 },
  };
}

function initializeCores(): CoreAdapter[] {
  return CORE_DEFINITIONS.map((def, idx) => {
    const isActive = idx < 3;
    return {
      ...def,
      status: isActive ? 'connected' as CoreStatus : 'standby' as CoreStatus,
      priority: 9 - idx,
      health: generateHealth(def.id, isActive),
      lastChecked: Date.now() - Math.floor(Math.random() * 15000),
      blockEvents24h: isActive ? Math.floor(Math.random() * 2) : Math.floor(Math.random() * 8),
    };
  });
}

function computeScore(core: CoreAdapter, rlWeight: number): number {
  if (core.status === 'error' || core.health.blocked) return 0;
  const latencyScore = Math.max(0, 100 - core.health.latency);
  const lossScore = Math.max(0, 100 - core.health.packetLoss * 20);
  const dnsScore = core.health.dnsLeak ? 0 : 100;
  const dpiScore = Math.max(0, 100 - core.health.dpiExposure * 2);
  const blockScore = Math.max(0, 100 - core.blockEvents24h * 15);
  const rlScore = rlWeight * 100;
  return Math.round(
    latencyScore * 0.25 + lossScore * 0.2 + dnsScore * 0.15 +
    dpiScore * 0.2 + blockScore * 0.1 + rlScore * 0.1
  );
}

function computeUCB(rewardHistory: number[], alpha: number, totalPulls: number): { exploitation: number; exploration: number; total: number } {
  const n = rewardHistory.length;
  if (n === 0) return { exploitation: 0.5, exploration: alpha * Math.sqrt(Math.log(totalPulls + 1) / 1), total: 0.5 + alpha };
  const avgReward = rewardHistory.reduce((a, b) => a + b, 0) / n;
  const exploration = alpha * Math.sqrt(Math.log(totalPulls + 1) / n);
  return { exploitation: Math.round(avgReward * 1000) / 1000, exploration: Math.round(exploration * 1000) / 1000, total: Math.round((avgReward + exploration) * 1000) / 1000 };
}

interface UnifiedShieldStore {
  connected: boolean;
  cores: CoreAdapter[];
  orchestrator: AIOrchestratorState;
  routing: TrafficRoutingState;
  ota: OTAUpdateState;
  dpiResults: DPITestResult[];
  stats: ConnectionStats;
  activeTab: string;
  activePlatform: string;
  rewardHistory: Record<string, number[]>;
  totalPulls: number;
  killSwitch: KillSwitchState;
  autoReconnect: AutoReconnectState;
  connectionLogs: ConnectionLogEntry[];
  threatIntel: ThreatIntelState;
  advancedSettings: AdvancedSettings;
  networkAnalyzer: NetworkAnalyzerState;
  geoRouter: GeoRouterState;
  securityAudit: SecurityAuditState;

  setConnected: (val: boolean) => void;
  setActiveTab: (tab: string) => void;
  setActivePlatform: (platform: string) => void;
  toggleConnection: () => void;
  updateCoreHealth: () => void;
  switchCore: (coreId: string) => void;
  setRoutingMode: (mode: 'full-vpn' | 'split-tunnel' | 'selective') => void;
  toggleIranBypass: () => void;
  setDnsMode: (mode: 'doh' | 'dot' | 'plain') => void;
  setDnsProvider: (provider: string) => void;
  runDPITest: () => void;
  performAIOrchestration: () => void;
  toggleKillSwitch: () => void;
  toggleNetworkLock: () => void;
  attemptReconnect: () => void;
  resetAutoReconnect: () => void;
  addLog: (entry: Omit<ConnectionLogEntry, 'id' | 'timestamp'>) => void;
  updateNetworkStats: () => void;
  selectServerCountry: (countryCode: string) => void;
  runSecurityAudit: () => void;
}

const INITIAL_REWARD_HISTORY: Record<string, number[]> = {
  'hiddify': [0.8, 0.9, 0.7, 0.85, 0.9],
  'xray-gfw': [0.95, 0.92, 0.88, 0.9, 0.93],
  'sing-box': [0.75, 0.8, 0.82, 0.78, 0.85],
  'amneziavpn': [0.7, 0.65, 0.8, 0.72, 0.68],
  'defyxvpn': [0.6, 0.65, 0.7, 0.55, 0.62],
  'moav': [0.55, 0.6, 0.5, 0.58, 0.52],
  'lantern': [0.45, 0.5, 0.48, 0.42, 0.47],
  'mahsang': [0.88, 0.92, 0.85, 0.9, 0.87],
  'psiphon': [0.35, 0.3, 0.4, 0.32, 0.28],
};

const UCB_ALPHAS: Record<string, number> = {
  'hiddify': 1.5, 'xray-gfw': 1.5, 'sing-box': 1.5,
  'amneziavpn': 2.0, 'defyxvpn': 1.5, 'moav': 1.5,
  'lantern': 1.5, 'mahsang': 1.5, 'psiphon': 0.5,
};

const INITIAL_CONNECTION_LOGS: ConnectionLogEntry[] = [
  { id: 'log-1', timestamp: Date.now() - 3600000, type: 'connect', message: 'Connected to xray-gfw core', messageFa: 'اتصال به هسته ایکس‌ری GFW برقرار شد', coreId: 'xray-gfw' },
  { id: 'log-2', timestamp: Date.now() - 3500000, type: 'switch', message: 'Shadow connection established with mahsang', messageFa: 'اتصال سایه با مهساان‌جی برقرار شد', coreId: 'mahsang' },
  { id: 'log-3', timestamp: Date.now() - 3000000, type: 'dpi-detect', message: 'DPI signature TLS-ClientHello-Reset detected', messageFa: 'امضای DPI بازنشانی ClientHello TLS شناسایی شد', details: { signature: 'TLS-ClientHello-Reset' } },
  { id: 'log-4', timestamp: Date.now() - 2500000, type: 'block', message: 'hiddify core blocked, auto-switching', messageFa: 'هسته هیدیفای مسدود شد، تعویض خودکار', coreId: 'hiddify' },
  { id: 'log-5', timestamp: Date.now() - 2000000, type: 'reconnect', message: 'Reconnected via mahsang core', messageFa: 'اتصال مجدد از طریق هسته مهساان‌جی برقرار شد', coreId: 'mahsang' },
  { id: 'log-6', timestamp: Date.now() - 1500000, type: 'update', message: 'iran-block-signatures updated to 2026.05.23-r2', messageFa: 'امضاهای مسدودیت ایران به ۲۰۲۶.۰۵.۲۳-r2 به‌روز شد' },
  { id: 'log-7', timestamp: Date.now() - 1000000, type: 'connect', message: 'Shadow connection established with hiddify', messageFa: 'اتصال سایه با هیدیفای برقرار شد', coreId: 'hiddify' },
  { id: 'log-8', timestamp: Date.now() - 500000, type: 'error', message: 'DNS leak detected, switching to DoH', messageFa: 'نشت DNS شناسایی شد، تعویض به DoH', details: { provider: 'cloudflare' } },
];

const INITIAL_THREATS: ThreatEntry[] = [
  { id: 'threat-1', type: 'DPI Deep Packet Inspection', typeFa: 'بازرسی عمیق بسته‌ها (DPI)', severity: 'high', description: 'Active TLS SNI filtering detected on current ISP', descriptionFa: 'فیلترینگ SNI فعال روی ISP فعلی شناسایی شد', detectedAt: Date.now() - 7200000, mitigated: true, countermeasure: 'VLESS Reality + XTLS', countermeasureFa: 'VLESS Reality + XTLS' },
  { id: 'threat-2', type: 'DNS Poisoning', typeFa: 'مسمومیت DNS', severity: 'critical', description: 'DNS responses being tampered with', descriptionFa: 'پاسخ‌های DNS دستکاری می‌شوند', detectedAt: Date.now() - 5400000, mitigated: true, countermeasure: 'DNS over HTTPS (DoH)', countermeasureFa: 'DNS over HTTPS (DoH)' },
  { id: 'threat-3', type: 'IP Blocking', typeFa: 'مسدودسازی IP', severity: 'medium', description: 'Several VPN server IPs blocked', descriptionFa: 'چندین IP سرور VPN مسدود شده', detectedAt: Date.now() - 3600000, mitigated: false, countermeasure: 'Domain fronting via CDN', countermeasureFa: 'فرانتینگ دامنه از طریق CDN' },
  { id: 'threat-4', type: 'Protocol Fingerprinting', typeFa: 'اثر انگشت پروتکل', severity: 'high', description: 'WireGuard handshake pattern detected by DPI', descriptionFa: 'الگوی دست‌دهی WireGuard توسط DPI شناسایی شد', detectedAt: Date.now() - 1800000, mitigated: true, countermeasure: 'AmneziaWG junk packets', countermeasureFa: 'بسته‌های جونک آمنزیاوی‌جی' },
  { id: 'threat-5', type: 'Null Routing', typeFa: 'مسیریابی صفر', severity: 'medium', description: 'Silent packet dropping on specific routes', descriptionFa: 'رها کردن بی‌صدا بسته‌ها در مسیرهای خاص', detectedAt: Date.now() - 900000, mitigated: false, countermeasure: 'Multi-path routing', countermeasureFa: 'مسیریابی چندمسیره' },
];

export const useUnifiedShieldStore = create<UnifiedShieldStore>((set, get) => ({
  connected: false,
  cores: initializeCores(),
  activeTab: 'dashboard',
  activePlatform: 'android',
  rewardHistory: INITIAL_REWARD_HISTORY,
  totalPulls: 47,

  orchestrator: {
    activeCoreId: 'xray-gfw',
    shadowConnections: ['mahsang', 'hiddify'],
    scoringMatrix: {},
    ucbScores: {},
    predictionState: {
      imminentBlockRisk: 12,
      predictedBlockCore: null,
      proactiveSwitchRecommended: false,
    },
    rlWeights: Object.fromEntries(CORE_DEFINITIONS.map(d => [d.id, [0.5, 0.3, 0.2, 0.6, 0.4]])),
    learningRate: 0.01,
    totalSwitches: 47,
    successfulSwitches: 44,
    averageSwitchTime: 1.3,
    detectedISP: 'irancell',
    detectedISPFa: 'ایرانسل',
    ispRuleApplied: 'irancell',
  },

  routing: {
    mode: 'split-tunnel',
    iranIpBypass: true,
    dnsMode: 'doh',
    dnsProviders: DNS_PROVIDERS.map(p => p.id),
    activeDnsProvider: 'cloudflare',
    ipv6Enabled: true,
    p2pRouting: true,
    splitRules: [
      { id: '1', app: 'Telegram', appFa: 'تلگرام', route: 'vpn', enabled: true },
      { id: '2', app: 'WhatsApp', appFa: 'واتساپ', route: 'vpn', enabled: true },
      { id: '3', app: 'Instagram', appFa: 'اینستاگرام', route: 'vpn', enabled: true },
      { id: '4', app: 'YouTube', appFa: 'یوتیوب', route: 'vpn', enabled: true },
      { id: '5', app: 'Twitter/X', appFa: 'توییتر/ایکس', route: 'vpn', enabled: true },
      { id: '6', app: 'Banking Apps', appFa: 'اپلیکیشن‌های بانکی', route: 'direct', enabled: true },
      { id: '7', app: 'Iranian Sites', appFa: 'سایت‌های ایرانی', route: 'direct', enabled: true },
      { id: '8', app: 'Tor Browser', appFa: 'مرورگر تور', route: 'vpn', enabled: true },
      { id: '9', app: 'Signal', appFa: 'سیگنال', route: 'vpn', enabled: true },
      { id: '10', app: 'Google Play', appFa: 'گوگل‌پلی', route: 'vpn', enabled: false },
    ],
  },

  ota: {
    lastCheck: Date.now() - 3600000,
    nextCheck: Date.now() + 18000000,
    updates: [
      {
        id: 'upd-1', type: 'core-binary', target: 'GFW-knocker/Xray-core',
        version: 'v25.8.3-mahsa-r1', currentVersion: 'v25.8.3-mahsa-r1',
        size: 5200000, deltaPatch: true,
        signature: 'sha256:a1b2c3d4e5f6g7h8', sha256: 'a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0',
        status: 'installed', githubReleaseUrl: 'https://api.github.com/repos/GFW-knocker/Xray-core/releases/latest',
      },
      {
        id: 'upd-2', type: 'block-db', target: 'iran-block-signatures',
        version: '2026.05.23-r2', currentVersion: '2026.05.20-r1',
        size: 256000, deltaPatch: false,
        signature: 'sha256:f6e5d4c3b2a1', sha256: 'f6e5d4c3b2a1z0y9x8w7v6u5t4s3r2q1p0o9n8m7',
        status: 'available', githubReleaseUrl: '',
      },
      {
        id: 'upd-3', type: 'ai-weights', target: 'ucb-mab-model',
        version: '3.1.0', currentVersion: '3.0.8',
        size: 1280000, deltaPatch: true,
        signature: 'sha256:1a2b3c4d5e6f', sha256: '1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t',
        status: 'available', githubReleaseUrl: '',
      },
      {
        id: 'upd-4', type: 'node-list', target: 'hiddify-nodes',
        version: '2026.05.23', currentVersion: '2026.05.22',
        size: 64000, deltaPatch: false,
        signature: 'sha256:9z8y7x6w5v4', sha256: '9z8y7x6w5v4u3t2s1r0q9p8o7n6m5l4k3j2i1h0g',
        status: 'installed', githubReleaseUrl: 'https://api.github.com/repos/hiddify/hiddify-core/releases/latest',
      },
      {
        id: 'upd-5', type: 'core-binary', target: 'DefyxVPN',
        version: 'v5.2.8', currentVersion: 'v5.2.8',
        size: 3800000, deltaPatch: false,
        signature: 'sha256:d4e5f6g7h8i9', sha256: 'd4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3',
        status: 'installed', githubReleaseUrl: 'https://api.github.com/repos/UnboundTechCo/defyxVPN/releases/latest',
      },
    ],
    autoUpdate: true,
    rollbackEnabled: true,
    sha256Verification: true,
    checkIntervalHours: 6,
  },

  dpiResults: [],

  stats: {
    totalUptime: 259200,
    totalDataTransferred: { up: 12.4, down: 87.6 },
    coresUsed: 5,
    switchesPerformed: 47,
    blockEventsAvoided: 23,
    currentSpeed: { up: 34, down: 156 },
    activePlatform: 'android',
  },

  killSwitch: {
    enabled: true,
    blockAllOnDisconnect: true,
    allowedApps: [],
    networkLock: true,
  },

  autoReconnect: {
    enabled: true,
    maxRetries: 10,
    retryCount: 0,
    retryInterval: 3000,
    exponentialBackoff: true,
    lastReconnectAttempt: 0,
    reconnectStatus: 'idle',
  },

  connectionLogs: INITIAL_CONNECTION_LOGS,

  threatIntel: {
    activeThreats: INITIAL_THREATS,
    lastScan: Date.now() - 1800000,
    threatLevel: 'high',
    dpiPatternsUpdated: '2026.05.23-r2',
    blockedDomainsCount: 1247,
    activeCountermeasures: ['VLESS Reality + XTLS', 'DNS over HTTPS (DoH)', 'AmneziaWG junk packets'],
  },

  advancedSettings: {
    language: 'fa',
    theme: 'dark',
    startOnBoot: true,
    autoConnectOnLaunch: true,
    notifications: true,
    stealthMode: false,
    debugMode: false,
    connectionTimeout: 15000,
    mtuSize: 1500,
  },

  networkAnalyzer: buildInitialNetworkAnalyzerState(),

  geoRouter: buildInitialGeoRouterState('irancell'),

  securityAudit: buildInitialSecurityAuditState(
    false,
    'xray-gfw',
    'doh',
    true,
    true,
    true,
  ),

  setConnected: (val) => set({ connected: val }),
  setActiveTab: (tab) => set({ activeTab: tab }),
  setActivePlatform: (platform) => set({ activePlatform: platform }),

  toggleConnection: () => {
    const { connected, cores } = get();
    if (!connected) {
      const updatedCores = cores.map((c, i) => {
        if (i === 0) return { ...c, status: 'connected' as CoreStatus, health: generateHealth(c.id, true) };
        if (i === 1) return { ...c, status: 'connected' as CoreStatus, health: generateHealth(c.id, true) };
        if (i === 2) return { ...c, status: 'standby' as CoreStatus, health: generateHealth(c.id, true) };
        return { ...c, status: 'standby' as CoreStatus, health: generateHealth(c.id, false) };
      });
      set({
        connected: true,
        cores: updatedCores,
        orchestrator: { ...get().orchestrator, activeCoreId: 'xray-gfw', shadowConnections: ['mahsang', 'hiddify'] },
        stats: { ...get().stats, currentSpeed: { up: 34, down: 156 } },
      });
    } else {
      const updatedCores = cores.map(c => ({
        ...c, status: 'disconnected' as CoreStatus, health: generateHealth(c.id, false),
      }));
      set({
        connected: false, cores: updatedCores,
        orchestrator: { ...get().orchestrator, activeCoreId: '', shadowConnections: [] },
        stats: { ...get().stats, currentSpeed: { up: 0, down: 0 } },
      });
    }
  },

  updateCoreHealth: () => {
    const { cores, orchestrator, connected } = get();
    if (!connected) return;

    const updatedCores = cores.map(c => {
      const isActive = c.id === orchestrator.activeCoreId || orchestrator.shadowConnections.includes(c.id);
      const prevHealth = c.health;
      const newBlocked = Math.random() < (isActive ? 0.04 : 0.02);
      const updatedHealth = {
        ...prevHealth,
        latency: Math.max(15, Math.round(prevHealth.latency * (1 + (Math.random() * 0.1 - 0.05)))),
        packetLoss: Math.max(0, Math.round((prevHealth.packetLoss + Math.random() * 0.5 - 0.25) * 100) / 100),
        dpiExposure: Math.max(0, Math.round((prevHealth.dpiExposure + Math.random() * 2 - 1) * 10) / 10),
        blocked: prevHealth.blocked ? Math.random() > 0.3 : newBlocked,
        bandwidth: isActive ? {
          up: Math.max(5, Math.round(prevHealth.bandwidth.up * (1 + (Math.random() * 0.2 - 0.1)))),
          down: Math.max(10, Math.round(prevHealth.bandwidth.down * (1 + (Math.random() * 0.2 - 0.1)))),
        } : { up: 0, down: 0 },
        uptime: isActive ? prevHealth.uptime + 15 : prevHealth.uptime,
      };
      // Sync status with health.blocked so computeScore and UI stay consistent
      let newStatus = c.status;
      if (updatedHealth.blocked && c.status === 'connected') newStatus = 'error' as CoreStatus;
      if (!updatedHealth.blocked && c.status === 'error') newStatus = 'standby' as CoreStatus;
      return {
        ...c,
        status: newStatus,
        health: updatedHealth,
        lastChecked: Date.now(),
      };
    });

    const scoringMatrix: Record<string, number> = {};
    for (const core of updatedCores) {
      const rlWeight = orchestrator.rlWeights[core.id]?.[0] ?? 0.5;
      scoringMatrix[core.id] = computeScore(core, rlWeight);
    }

    set({ cores: updatedCores, orchestrator: { ...orchestrator, scoringMatrix } });
  },

  switchCore: (coreId) => {
    const { cores, orchestrator, connected, rewardHistory, totalPulls } = get();
    if (!connected) return;

    const targetCore = cores.find(c => c.id === coreId);
    if (!targetCore || targetCore.health.blocked) return;

    const oldActive = orchestrator.activeCoreId;
    const newShadows = [oldActive, ...orchestrator.shadowConnections.filter(id => id !== coreId)].slice(0, 2);

    const updatedCores = cores.map(c => {
      if (c.id === coreId) return { ...c, status: 'connected' as CoreStatus };
      if (c.id === oldActive) return { ...c, status: 'standby' as CoreStatus };
      if (newShadows.includes(c.id)) return { ...c, status: 'standby' as CoreStatus };
      return c;
    });

    const success = Math.random() > 0.05;
    const reward = success ? (1.0 - Math.min(targetCore.health.latency / 5000, 0.9)) : 0;
    const newHistory = { ...rewardHistory };
    newHistory[coreId] = [...(newHistory[coreId] ?? []), reward].slice(-100);

    set({
      cores: updatedCores,
      rewardHistory: newHistory,
      totalPulls: totalPulls + 1,
      orchestrator: {
        ...orchestrator,
        activeCoreId: coreId,
        shadowConnections: newShadows,
        totalSwitches: orchestrator.totalSwitches + 1,
        successfulSwitches: orchestrator.successfulSwitches + (success ? 1 : 0),
      },
    });
  },

  setRoutingMode: (mode) => set({ routing: { ...get().routing, mode } }),
  toggleIranBypass: () => set({ routing: { ...get().routing, iranIpBypass: !get().routing.iranIpBypass } }),
  setDnsMode: (mode) => set({ routing: { ...get().routing, dnsMode: mode } }),
  setDnsProvider: (provider) => set({ routing: { ...get().routing, activeDnsProvider: provider } }),

  runDPITest: () => {
    const { cores } = get();
    const latencyBases: Record<string, number> = {
      'hiddify': 85, 'xray-gfw': 62, 'sing-box': 73, 'amneziavpn': 91,
      'defyxvpn': 105, 'moav': 118, 'lantern': 142, 'mahsang': 79, 'psiphon': 156,
    };
    const sigEntries = IRAN_DPI_SIGNATURES;
    const results: DPITestResult[] = cores.map(core => {
      const connected = Math.random() > 0.15;
      const bypassLevel = connected ? (Math.random() > 0.3 ? 'full' : 'partial') : 'none';
      const sigEntry = sigEntries[Math.floor(Math.random() * sigEntries.length)];
      return {
        coreId: core.id, coreName: core.name, coreNameFa: core.nameFa,
        connected, latency: connected ? Math.round((latencyBases[core.id] ?? 100) + Math.random() * 30) : 0,
        protocol: core.capabilities[0], bypassLevel,
        dpiSignature: sigEntry.signature, dpiSignatureFa: sigEntry.descriptionFa,
        timestamp: Date.now(),
      };
    });
    set({ dpiResults: results });
  },

  performAIOrchestration: () => {
    const { cores, orchestrator, connected, rewardHistory, totalPulls } = get();
    if (!connected) return;

    const scoringMatrix: Record<string, number> = {};
    const ucbScores: Record<string, { exploitation: number; exploration: number; total: number }> = {};
    let bestCore = orchestrator.activeCoreId;
    let bestScore = 0;

    for (const core of cores) {
      const rlWeight = orchestrator.rlWeights[core.id]?.[0] ?? 0.5;
      const score = computeScore(core, rlWeight);
      scoringMatrix[core.id] = score;

      const alpha = UCB_ALPHAS[core.id] ?? 1.5;
      ucbScores[core.id] = computeUCB(rewardHistory[core.id] ?? [], alpha, totalPulls);

      if (score > bestScore && !core.health.blocked && core.status !== 'error') {
        bestScore = score;
        bestCore = core.id;
      }
    }

    const currentScore = scoringMatrix[orchestrator.activeCoreId] ?? 0;
    const shouldSwitch = bestCore !== orchestrator.activeCoreId && (bestScore - currentScore) > 15;

    const latencySpikes = cores.filter(c => c.health.latency > 150).map(c => c.id);
    const imminentBlockRisk = Math.min(100, latencySpikes.length * 20);
    const predictedBlockCore = latencySpikes.length > 0 ? latencySpikes[0] : null;
    const proactiveSwitchRecommended = imminentBlockRisk > 40;

    const newRlWeights = { ...orchestrator.rlWeights };
    for (const core of cores) {
      const weights = [...(newRlWeights[core.id] ?? [0.5, 0.3, 0.2, 0.6, 0.4])];
      if (core.id === orchestrator.activeCoreId) {
        weights[0] = Math.min(1, weights[0] + orchestrator.learningRate * (core.health.blocked ? -5 : 1));
      } else {
        weights[0] = Math.max(0, weights[0] - orchestrator.learningRate * 0.5);
      }
      newRlWeights[core.id] = weights;
    }

    const ispRule = ISP_RULES.find(r => r.id === orchestrator.detectedISP) ?? ISP_RULES[1];

    if (shouldSwitch || (proactiveSwitchRecommended && predictedBlockCore === orchestrator.activeCoreId)) {
      get().switchCore(bestCore);
    }

    // Re-read orchestrator from store after possible switchCore mutation to avoid stale state overwrite
    const latestOrchestrator = get().orchestrator;
    set({
      orchestrator: {
        ...latestOrchestrator,
        scoringMatrix,
        ucbScores,
        predictionState: { imminentBlockRisk, predictedBlockCore, proactiveSwitchRecommended },
        rlWeights: newRlWeights,
        ispRuleApplied: ispRule.id,
      },
    });
  },

  toggleKillSwitch: () => {
    const { killSwitch } = get();
    set({ killSwitch: { ...killSwitch, enabled: !killSwitch.enabled } });
  },

  toggleNetworkLock: () => {
    const { killSwitch } = get();
    set({ killSwitch: { ...killSwitch, networkLock: !killSwitch.networkLock } });
  },

  resetAutoReconnect: () => {
    const { autoReconnect } = get();
    set({ autoReconnect: { ...autoReconnect, retryCount: 0, reconnectStatus: 'idle' } });
  },

  attemptReconnect: () => {
    const { autoReconnect, connected } = get();
    if (connected || !autoReconnect.enabled) return;

    if (autoReconnect.retryCount >= autoReconnect.maxRetries) {
      set({ autoReconnect: { ...autoReconnect, reconnectStatus: 'failed' } });
      return;
    }

    set({ autoReconnect: { ...autoReconnect, reconnectStatus: 'reconnecting', lastReconnectAttempt: Date.now() } });

    const delay = autoReconnect.exponentialBackoff
      ? autoReconnect.retryInterval * Math.pow(2, autoReconnect.retryCount)
      : autoReconnect.retryInterval;

    setTimeout(() => {
      const success = Math.random() > 0.3;
      if (success) {
        get().toggleConnection();
        set({
          autoReconnect: {
            ...get().autoReconnect,
            reconnectStatus: 'connected',
            retryCount: 0,
          },
        });
      } else {
        set({
          autoReconnect: {
            ...get().autoReconnect,
            reconnectStatus: 'reconnecting',
            retryCount: get().autoReconnect.retryCount + 1,
          },
        });
      }
    }, Math.min(delay, 30000));
  },

  addLog: (entry) => {
    const { connectionLogs } = get();
    const newLog: ConnectionLogEntry = {
      ...entry,
      id: `log-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      timestamp: Date.now(),
    };
    set({ connectionLogs: [newLog, ...connectionLogs].slice(0, 200) });
  },

  updateNetworkStats: () => {
    const { networkAnalyzer, connected } = get();
    const updatedAnalyzer = updateNetworkMonitoring(networkAnalyzer, connected);
    set({ networkAnalyzer: updatedAnalyzer });
  },

  selectServerCountry: (countryCode: string) => {
    const { geoRouter } = get();
    const updatedGeoRouter = geoSelectServerCountry(geoRouter, countryCode);
    set({ geoRouter: updatedGeoRouter });
  },

  runSecurityAudit: () => {
    const { securityAudit, connected, orchestrator, routing, killSwitch } = get();
    const updatedAudit = runFullSecurityAudit(
      securityAudit,
      connected,
      orchestrator.activeCoreId,
      routing.dnsMode,
      killSwitch.enabled,
      killSwitch.networkLock,
      routing.ipv6Enabled,
    );
    set({ securityAudit: updatedAudit });
  },
}));
