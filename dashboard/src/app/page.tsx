'use client';

import React, { useEffect, useCallback, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Shield, ShieldCheck, ShieldAlert, Activity, Zap, Globe, Brain,
  Server, RefreshCw, WifiOff,
  ArrowUpCircle, ArrowDownCircle, Clock, AlertTriangle, CheckCircle2,
  XCircle, Eye, Lock, Settings, BarChart3, Radio,
  Layers, CloudDownload, Cpu, Gauge, ArrowRightLeft, Search,
  MonitorSmartphone, Network, ShieldHalf, Timer, TrendingUp,
  Signal, Database, GitBranch, FileCheck,
  RotateCcw, TestTube, Bug, Scan,
  Smartphone, Monitor, Terminal, Apple, Router, Laptop,
  MapPin, AlertOctagon, Fingerprint, Power,
  Ban, ShieldOff, Siren,
  FileText, Sun, Moon,
  Trash2, Plus,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Switch } from '@/components/ui/switch';
import {
  AreaChart, Area, BarChart, Bar, LineChart, Line,
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  PieChart, Pie, Cell, RadarChart, Radar, PolarGrid,
  PolarAngleAxis, PolarRadiusAxis, Legend,
} from 'recharts';
import { useUnifiedShieldStore } from '@/lib/unified-shield-store';
import {
  PROTOCOL_LABELS, DNS_PROVIDERS, COUNTRY_SERVERS,
  PLATFORMS, ISP_RULES, IRAN_DPI_SIGNATURES,
  type CoreAdapter, type DPITestResult, type ProtocolType,
  type ConnectionLogEntry, type ThreatEntry,
} from '@/lib/unified-shield-types';
import NetworkAnalyzerPanel from '@/components/network-analyzer-panel';
import GeoRouterPanel from '@/components/geo-router-panel';
import SecurityAuditPanel from '@/components/security-audit-panel';
import RealtimeStatusBar from '@/components/realtime-status-bar';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '۰ بایت';
  const units = ['بایت', 'کیلوبایت', 'مگابایت', 'گیگابایت', 'ترابایت'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days} روز ${hours} ساعت`;
  if (hours > 0) return `${hours} ساعت ${mins} دقیقه`;
  return `${mins} دقیقه`;
}

function toPersianNum(n: number | string): string {
  const persianDigits = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];
  return String(n).replace(/\d/g, d => persianDigits[parseInt(d)]);
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'connected': return 'text-emerald-400';
    case 'connecting': return 'text-yellow-400';
    case 'standby': return 'text-slate-400';
    case 'error': return 'text-red-400';
    case 'disconnected': return 'text-slate-500';
    default: return 'text-slate-400';
  }
}

function getStatusLabel(status: string): string {
  switch (status) {
    case 'connected': return 'متصل';
    case 'connecting': return 'در حال اتصال';
    case 'standby': return 'آماده';
    case 'error': return 'خطا';
    case 'disconnected': return 'قطع';
    default: return 'نامشخص';
  }
}

function getStatusBg(status: string): string {
  switch (status) {
    case 'connected': return 'bg-emerald-500/20 border-emerald-500/30';
    case 'connecting': return 'bg-yellow-500/20 border-yellow-500/30';
    case 'standby': return 'bg-slate-500/20 border-slate-500/30';
    case 'error': return 'bg-red-500/20 border-red-500/30';
    case 'disconnected': return 'bg-slate-500/10 border-slate-500/20';
    default: return 'bg-slate-500/20 border-slate-500/30';
  }
}

function getLogTypeColor(type: ConnectionLogEntry['type']): string {
  switch (type) {
    case 'connect': return 'text-emerald-400';
    case 'disconnect': return 'text-red-400';
    case 'switch': return 'text-violet-400';
    case 'block': return 'text-red-400';
    case 'reconnect': return 'text-cyan-400';
    case 'dpi-detect': return 'text-amber-400';
    case 'update': return 'text-sky-400';
    case 'error': return 'text-red-400';
    default: return 'text-slate-400';
  }
}

function getLogTypeLabel(type: ConnectionLogEntry['type']): string {
  switch (type) {
    case 'connect': return 'اتصال';
    case 'disconnect': return 'قطع';
    case 'switch': return 'تعویض';
    case 'block': return 'مسدودیت';
    case 'reconnect': return 'اتصال مجدد';
    case 'dpi-detect': return 'شناسایی DPI';
    case 'update': return 'به‌روزرسانی';
    case 'error': return 'خطا';
    default: return 'نامشخص';
  }
}

function getLogTypeBg(type: ConnectionLogEntry['type']): string {
  switch (type) {
    case 'connect': return 'bg-emerald-500/10 border-emerald-500/30';
    case 'disconnect': return 'bg-red-500/10 border-red-500/30';
    case 'switch': return 'bg-violet-500/10 border-violet-500/30';
    case 'block': return 'bg-red-500/10 border-red-500/30';
    case 'reconnect': return 'bg-cyan-500/10 border-cyan-500/30';
    case 'dpi-detect': return 'bg-amber-500/10 border-amber-500/30';
    case 'update': return 'bg-sky-500/10 border-sky-500/30';
    case 'error': return 'bg-red-500/10 border-red-500/30';
    default: return 'bg-slate-500/10 border-slate-500/30';
  }
}

function getSeverityColor(severity: string): string {
  switch (severity) {
    case 'critical': return 'text-red-400';
    case 'high': return 'text-orange-400';
    case 'medium': return 'text-yellow-400';
    case 'low': return 'text-emerald-400';
    default: return 'text-slate-400';
  }
}

function getSeverityBg(severity: string): string {
  switch (severity) {
    case 'critical': return 'bg-red-500/10 border-red-500/30';
    case 'high': return 'bg-orange-500/10 border-orange-500/30';
    case 'medium': return 'bg-yellow-500/10 border-yellow-500/30';
    case 'low': return 'bg-emerald-500/10 border-emerald-500/30';
    default: return 'bg-slate-500/10 border-slate-500/30';
  }
}

function getSeverityLabel(severity: string): string {
  switch (severity) {
    case 'critical': return 'بحرانی';
    case 'high': return 'بالا';
    case 'medium': return 'متوسط';
    case 'low': return 'پایین';
    default: return 'نامشخص';
  }
}

// ──────────────────────────────────────────────
// Connect Button
// ──────────────────────────────────────────────
function ConnectButton() {
  const { connected, toggleConnection } = useUnifiedShieldStore();
  const [connecting, setConnecting] = useState(false);

  const handleConnect = useCallback(() => {
    if (!connected) {
      setConnecting(true);
      setTimeout(() => { toggleConnection(); setConnecting(false); }, 2000);
    } else { toggleConnection(); }
  }, [connected, toggleConnection]);

  return (
    <div className="flex flex-col items-center gap-4">
      <motion.button
        onClick={handleConnect} disabled={connecting}
        className="relative w-36 h-36 rounded-full flex items-center justify-center transition-all duration-500"
        whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.95 }}
        style={{ background: connected ? 'radial-gradient(circle, rgba(16,185,129,0.3) 0%, rgba(16,185,129,0.1) 50%, transparent 70%)' : 'radial-gradient(circle, rgba(99,102,241,0.3) 0%, rgba(99,102,241,0.1) 50%, transparent 70%)' }}
      >
        {connected && <motion.div className="absolute inset-0 rounded-full border-2 border-emerald-400/50" animate={{ scale: [1, 1.3, 1], opacity: [0.5, 0, 0.5] }} transition={{ duration: 2, repeat: Infinity }} />}
        <div className={`w-28 h-28 rounded-full flex items-center justify-center border-2 transition-all duration-500 ${connected ? 'bg-emerald-500/20 border-emerald-400 shadow-lg shadow-emerald-500/30' : connecting ? 'bg-yellow-500/20 border-yellow-400 shadow-lg shadow-yellow-500/30' : 'bg-slate-700/50 border-slate-500'}`}>
          {connecting ? <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}><RefreshCw className="w-12 h-12 text-yellow-400" /></motion.div> : connected ? <ShieldCheck className="w-12 h-12 text-emerald-400" /> : <Shield className="w-12 h-12 text-slate-400" />}
        </div>
      </motion.button>
      <div className="text-center">
        <p className={`text-xl font-bold ${connected ? 'text-emerald-400' : 'text-slate-300'}`}>{connecting ? 'در حال اتصال...' : connected ? 'متصل و ایمن' : 'اتصال با یک لمس'}</p>
        <p className="text-sm text-slate-500 mt-1">{connected ? 'یونیفایدشیلد از شما محافظت می‌کند' : 'برای اتصال خودکار لمس کنید'}</p>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Platform Selector
// ──────────────────────────────────────────────
function PlatformSelector() {
  const { activePlatform, setActivePlatform } = useUnifiedShieldStore();
  const platformIcons: Record<string, React.ReactNode> = {
    android: <Smartphone className="w-4 h-4" />,
    windows: <Monitor className="w-4 h-4" />,
    linux: <Terminal className="w-4 h-4" />,
    ios: <Apple className="w-4 h-4" />,
    openwrt: <Router className="w-4 h-4" />,
    macos: <Laptop className="w-4 h-4" />,
  };

  return (
    <div className="flex gap-2 flex-wrap justify-center">
      {PLATFORMS.map(p => (
        <button key={p.platform} onClick={() => setActivePlatform(p.platform)}
          className={`flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-all ${activePlatform === p.platform ? 'border-cyan-500/50 bg-cyan-500/10 text-cyan-300' : 'border-slate-700/30 bg-slate-800/30 text-slate-400 hover:bg-slate-800/50'}`}>
          <span className="text-base">{p.icon}</span>
          {p.nameFa}
        </button>
      ))}
    </div>
  );
}

// ──────────────────────────────────────────────
// Speed Monitor
// ──────────────────────────────────────────────
function SpeedMonitor() {
  const { stats, connected } = useUnifiedShieldStore();
  const [speedHistory, setSpeedHistory] = useState<Array<{ time: string; upload: number; download: number }>>([]);

  useEffect(() => {
    if (!connected) return;
    const interval = setInterval(() => {
      const now = new Date();
      const timeStr = `${toPersianNum(now.getHours())}:${toPersianNum(String(now.getMinutes()).padStart(2, '0'))}`;
      setSpeedHistory(prev => {
        const newEntry = { time: timeStr, upload: Math.max(5, stats.currentSpeed.up + (Math.random() * 20 - 10)), download: Math.max(20, stats.currentSpeed.down + (Math.random() * 60 - 30)) };
        return [...prev, newEntry].slice(-20);
      });
    }, 2000);
    return () => clearInterval(interval);
  }, [connected, stats.currentSpeed.up, stats.currentSpeed.down]);

  return (
    <Card className="bg-slate-800/50 border-slate-700/50">
      <CardHeader className="pb-2">
        <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Gauge className="w-5 h-5 text-cyan-400" />سرعت لحظه‌ای</CardTitle>
      </CardHeader>
      <CardContent>
        {connected ? (
          <div className="space-y-3">
            <div className="flex items-center justify-between"><div className="flex items-center gap-2"><ArrowUpCircle className="w-5 h-5 text-emerald-400" /><span className="text-slate-400 text-sm">آپلود</span></div><span className="text-emerald-400 font-bold text-lg">{toPersianNum(Math.round(stats.currentSpeed.up))} Mbps</span></div>
            <div className="flex items-center justify-between"><div className="flex items-center gap-2"><ArrowDownCircle className="w-5 h-5 text-cyan-400" /><span className="text-slate-400 text-sm">دانلود</span></div><span className="text-cyan-400 font-bold text-lg">{toPersianNum(Math.round(stats.currentSpeed.down))} Mbps</span></div>
            <div className="h-40 mt-2">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={speedHistory}>
                  <defs>
                    <linearGradient id="dlGrad" x1="0" y1="0" x2="0" y2="1"><stop offset="5%" stopColor="#06b6d4" stopOpacity={0.3} /><stop offset="95%" stopColor="#06b6d4" stopOpacity={0} /></linearGradient>
                    <linearGradient id="ulGrad" x1="0" y1="0" x2="0" y2="1"><stop offset="5%" stopColor="#10b981" stopOpacity={0.3} /><stop offset="95%" stopColor="#10b981" stopOpacity={0} /></linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                  <XAxis dataKey="time" tick={{ fill: '#94a3b8', fontSize: 10 }} />
                  <YAxis tick={{ fill: '#94a3b8', fontSize: 10 }} />
                  <Tooltip contentStyle={{ background: '#1e293b', border: '1px solid #475569', borderRadius: 8 }} labelStyle={{ color: '#e2e8f0' }} />
                  <Area type="monotone" dataKey="download" stroke="#06b6d4" fill="url(#dlGrad)" strokeWidth={2} name="دانلود" />
                  <Area type="monotone" dataKey="upload" stroke="#10b981" fill="url(#ulGrad)" strokeWidth={2} name="آپلود" />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </div>
        ) : (
          <div className="h-40 flex items-center justify-center text-slate-500"><div className="text-center"><WifiOff className="w-10 h-10 mx-auto mb-2 opacity-50" /><p>ابتدا متصل شوید</p></div></div>
        )}
      </CardContent>
    </Card>
  );
}

// ──────────────────────────────────────────────
// Core Card
// ──────────────────────────────────────────────
function CoreCard({ core, isActive, score }: { core: CoreAdapter; isActive: boolean; score: number }) {
  const { switchCore, connected } = useUnifiedShieldStore();

  return (
    <motion.div layout initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}
      className={`relative rounded-xl border p-4 transition-all duration-300 ${isActive ? 'border-emerald-500/40 bg-emerald-500/5 shadow-lg shadow-emerald-500/10' : core.health.blocked ? 'border-red-500/30 bg-red-500/5' : 'border-slate-700/50 bg-slate-800/30 hover:bg-slate-800/50'}`}>
      {isActive && <div className="absolute top-2 left-2"><motion.div className="w-3 h-3 rounded-full bg-emerald-400" animate={{ opacity: [1, 0.4, 1] }} transition={{ duration: 1.5, repeat: Infinity }} /></div>}
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2">
          <span className="text-2xl">{core.icon}</span>
          <div>
            <h4 className="text-slate-200 font-semibold text-sm">{core.nameFa}</h4>
            <p className="text-slate-500 text-xs">{core.version}</p>
          </div>
        </div>
        <Badge variant="outline" className={`text-xs ${getStatusBg(core.status)} ${getStatusColor(core.status)}`}>{getStatusLabel(core.status)}</Badge>
      </div>
      <p className="text-xs text-slate-500 mb-2">{core.roleFa}</p>
      <div className="grid grid-cols-2 gap-1.5 text-xs mb-2">
        <div className="flex items-center gap-1"><Timer className="w-3 h-3 text-slate-500" /><span className="text-slate-400">تأخیر:</span><span className={core.health.latency > 150 ? 'text-yellow-400' : 'text-emerald-400'}>{toPersianNum(core.health.latency)} ms</span></div>
        <div className="flex items-center gap-1"><Signal className="w-3 h-3 text-slate-500" /><span className="text-slate-400">افت:</span><span className={core.health.packetLoss > 5 ? 'text-red-400' : 'text-emerald-400'}>{toPersianNum(core.health.packetLoss)}٪</span></div>
        <div className="flex items-center gap-1"><Eye className="w-3 h-3 text-slate-500" /><span className="text-slate-400">DPI:</span><span className={core.health.dpiExposure > 30 ? 'text-red-400' : 'text-emerald-400'}>{toPersianNum(core.health.dpiExposure)}</span></div>
        <div className="flex items-center gap-1"><Shield className="w-3 h-3 text-slate-500" /><span className="text-slate-400">امتیاز:</span><span className="text-cyan-400 font-bold">{toPersianNum(score)}</span></div>
      </div>
      <div className="flex flex-wrap gap-1 mb-2">
        {core.capabilities.slice(0, 3).map(cap => <Badge key={cap} variant="secondary" className="text-[10px] bg-slate-700/50 text-slate-300">{PROTOCOL_LABELS[cap]?.nameFa ?? cap}</Badge>)}
        {core.capabilities.length > 3 && <Badge variant="secondary" className="text-[10px] bg-slate-700/50 text-slate-400">+{toPersianNum(core.capabilities.length - 3)}</Badge>}
      </div>
      <div className="flex flex-wrap gap-1 mb-2">
        {core.specialFeaturesFa.slice(0, 2).map((f, i) => <span key={i} className="text-[10px] text-slate-500 bg-slate-700/30 rounded px-1.5 py-0.5">{f}</span>)}
      </div>
      <div className="flex items-center justify-between">
        <div className="text-xs text-slate-500">مسدودی: <span className={core.blockEvents24h > 3 ? 'text-red-400' : 'text-slate-400'}>{toPersianNum(core.blockEvents24h)}</span></div>
        {connected && !isActive && !core.health.blocked && <Button size="sm" variant="ghost" className="h-7 text-xs text-cyan-400 hover:text-cyan-300 hover:bg-cyan-500/10" onClick={() => switchCore(core.id)}><ArrowRightLeft className="w-3 h-3 ml-1" />تعویض</Button>}
      </div>
      {core.health.blocked && <div className="mt-2 flex items-center gap-1 text-xs text-red-400 bg-red-500/10 rounded px-2 py-1"><ShieldAlert className="w-3 h-3" />مسدود — هوش مصنوعی در حال تعویض خودکار</div>}
    </motion.div>
  );
}

function CoreGrid() {
  const { cores, orchestrator } = useUnifiedShieldStore();
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
      {cores.map(core => <CoreCard key={core.id} core={core} isActive={core.id === orchestrator.activeCoreId} score={orchestrator.scoringMatrix[core.id] ?? 0} />)}
    </div>
  );
}

// ──────────────────────────────────────────────
// AI Orchestrator Panel with UCB
// ──────────────────────────────────────────────
function AIOrchestratorPanel() {
  const { orchestrator, cores, connected, rewardHistory } = useUnifiedShieldStore();
  const activeCore = cores.find(c => c.id === orchestrator.activeCoreId);
  const scoringData = cores.map(c => ({ name: c.nameFa, score: orchestrator.scoringMatrix[c.id] ?? 0, fill: c.color })).sort((a, b) => b.score - a.score);

  const ucbData = cores.map(c => {
    const ucb = orchestrator.ucbScores[c.id] ?? { exploitation: 0, exploration: 0, total: 0 };
    return { name: c.nameFa, بهره‌برداری: Math.round(ucb.exploitation * 100), اکتشاف: Math.round(ucb.exploration * 100), total: Math.round(ucb.total * 100) };
  }).sort((a, b) => b.total - a.total);

  const riskLevel = orchestrator.predictionState.imminentBlockRisk;
  const riskColor = riskLevel > 60 ? 'text-red-400' : riskLevel > 30 ? 'text-yellow-400' : 'text-emerald-400';
  const ispRule = ISP_RULES.find(r => r.id === orchestrator.ispRuleApplied);

  return (
    <div className="space-y-4">
      {/* ISP Detection & Active Core */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="bg-slate-800/50 border-slate-700/50">
          <CardHeader className="pb-2">
            <CardTitle className="text-slate-200 text-base flex items-center gap-2"><MapPin className="w-5 h-5 text-amber-400" />ISP شناسایی‌شده</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3 mb-3">
              <div className="w-12 h-12 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center">
                <Globe className="w-6 h-6 text-amber-400" />
              </div>
              <div>
                <p className="text-lg font-bold text-amber-300">{orchestrator.detectedISPFa}</p>
                <p className="text-xs text-slate-500">{ispRule?.name ?? 'نامشخص'}</p>
              </div>
            </div>
            {ispRule && (
              <div className="space-y-2 text-xs">
                <div><span className="text-slate-400">هسته‌های ترجیحی:</span><div className="flex gap-1 mt-1 flex-wrap">{ispRule.preferredCores.map(cId => { const c = cores.find(x => x.id === cId); return c ? <Badge key={cId} variant="outline" className="text-[10px] border-emerald-500/30 text-emerald-400">{c.nameFa}</Badge> : null; })}</div></div>
                <div><span className="text-slate-400">پنهان‌سازی بهترین:</span><div className="flex gap-1 mt-1 flex-wrap">{ispRule.bestObfuscationFa.map(o => <Badge key={o} variant="outline" className="text-[10px] border-cyan-500/30 text-cyan-400">{o}</Badge>)}</div></div>
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="bg-slate-800/50 border-slate-700/50">
          <CardHeader className="pb-2">
            <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Cpu className="w-5 h-5 text-violet-400" />هسته فعال</CardTitle>
          </CardHeader>
          <CardContent>
            {activeCore ? (
              <div className="space-y-2">
                <div className="flex items-center gap-3"><span className="text-3xl">{activeCore.icon}</span><div><p className="text-lg font-bold text-slate-200">{activeCore.nameFa}</p><p className="text-xs text-slate-500">{activeCore.version}</p></div></div>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div className="bg-slate-700/30 rounded p-2"><span className="text-slate-400 block">تأخیر</span><p className="text-emerald-400 font-bold">{toPersianNum(activeCore.health.latency)} ms</p></div>
                  <div className="bg-slate-700/30 rounded p-2"><span className="text-slate-400 block">پهنای باند</span><p className="text-cyan-400 font-bold">{toPersianNum(activeCore.health.bandwidth.down)} Mbps</p></div>
                </div>
              </div>
            ) : <p className="text-slate-500 text-sm">متصل نیست</p>}
          </CardContent>
        </Card>

        <Card className="bg-slate-800/50 border-slate-700/50">
          <CardHeader className="pb-2">
            <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Brain className="w-5 h-5 text-pink-400" />پیش‌بینی AI</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="bg-slate-700/30 rounded-lg p-3">
                <div className="flex items-center justify-between mb-2"><span className="text-slate-400 text-sm">خطر مسدودیت</span><span className={`font-bold ${riskColor}`}>{toPersianNum(riskLevel)}٪</span></div>
                <Progress value={riskLevel} className="h-2" />
              </div>
              {orchestrator.predictionState.proactiveSwitchRecommended && (
                <div className="flex items-center gap-2 text-xs bg-violet-500/10 border border-violet-500/20 rounded-lg px-3 py-2"><Zap className="w-4 h-4 text-violet-400" /><span className="text-violet-300">تعویض پیش‌دستانه فعال</span></div>
              )}
              <div className="grid grid-cols-3 gap-2 text-xs">
                <div className="bg-slate-700/30 rounded p-2 text-center"><span className="text-slate-400 block">تعویض‌ها</span><p className="text-cyan-400 font-bold">{toPersianNum(orchestrator.totalSwitches)}</p></div>
                <div className="bg-slate-700/30 rounded p-2 text-center"><span className="text-slate-400 block">موفق</span><p className="text-emerald-400 font-bold">{toPersianNum(orchestrator.successfulSwitches)}</p></div>
                <div className="bg-slate-700/30 rounded p-2 text-center"><span className="text-slate-400 block">زمان</span><p className="text-violet-400 font-bold">{toPersianNum(orchestrator.averageSwitchTime)}s</p></div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* UCB Multi-Armed Bandit */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Brain className="w-5 h-5 text-pink-400" />الگوریتم UCB (Multi-Armed Bandit)</CardTitle>
          <CardDescription className="text-slate-500 text-xs">بهره‌برداری vs اکتشاف — مدل یادگیری روی دستگاه، بدون وابستگی ابری</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={ucbData} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                <XAxis type="number" tick={{ fill: '#94a3b8', fontSize: 10 }} />
                <YAxis dataKey="name" type="category" tick={{ fill: '#e2e8f0', fontSize: 11 }} width={80} />
                <Tooltip contentStyle={{ background: '#1e293b', border: '1px solid #475569', borderRadius: 8 }} />
                <Bar dataKey="بهره‌برداری" stackId="a" fill="#10b981" fillOpacity={0.7} />
                <Bar dataKey="اکتشاف" stackId="a" fill="#8b5cf6" fillOpacity={0.7} radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
          <div className="flex justify-center gap-6 mt-2 text-xs">
            <div className="flex items-center gap-1"><div className="w-3 h-3 rounded bg-emerald-500/70" /><span className="text-slate-400">بهره‌برداری (Exploitation)</span></div>
            <div className="flex items-center gap-1"><div className="w-3 h-3 rounded bg-violet-500/70" /><span className="text-slate-400">اکتشاف (Exploration)</span></div>
          </div>
        </CardContent>
      </Card>

      {/* Scoring Matrix */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><BarChart3 className="w-5 h-5 text-amber-400" />ماتریس امتیازدهی هسته‌ها</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-56">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={scoringData} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                <XAxis type="number" domain={[0, 100]} tick={{ fill: '#94a3b8', fontSize: 11 }} />
                <YAxis dataKey="name" type="category" tick={{ fill: '#e2e8f0', fontSize: 12 }} width={80} />
                <Tooltip contentStyle={{ background: '#1e293b', border: '1px solid #475569', borderRadius: 8 }} />
                <Bar dataKey="score" radius={[0, 4, 4, 0]} name="امتیاز">{scoringData.map((entry, index) => <Cell key={`cell-${index}`} fill={entry.fill} fillOpacity={0.7} />)}</Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>

      {/* Shadow Connections */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Layers className="w-5 h-5 text-teal-400" />اتصالات سایه (تعویض فوری بدون قطعی)</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-3 flex-wrap">
            {orchestrator.shadowConnections.map(id => { const core = cores.find(c => c.id === id); if (!core) return null; return (
              <div key={id} className="flex items-center gap-2 bg-teal-500/10 border border-teal-500/20 rounded-lg px-3 py-2">
                <span>{core.icon}</span><div><p className="text-sm text-teal-300 font-medium">{core.nameFa}</p><p className="text-xs text-slate-500">{toPersianNum(core.health.latency)} ms</p></div>
                <motion.div className="w-2 h-2 rounded-full bg-teal-400 ml-1" animate={{ opacity: [1, 0.3, 1] }} transition={{ duration: 2, repeat: Infinity }} />
              </div>
            ); })}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// Protocol Stack Panel
// ──────────────────────────────────────────────
function ProtocolStackPanel() {
  const { cores, orchestrator } = useUnifiedShieldStore();
  const allProtocols = Object.entries(PROTOCOL_LABELS).map(([key, label]) => {
    const supportedCores = cores.filter(c => c.capabilities.includes(key as ProtocolType));
    const isActive = supportedCores.some(c => c.id === orchestrator.activeCoreId);
    return { key, ...label, supportedCores, isActive };
  }).sort((a, b) => a.priority - b.priority);

  const radarData = React.useMemo(() =>
    allProtocols.slice(0, 8).map((p, idx) => ({
      protocol: p.nameFa,
      cores: p.supportedCores.length,
      // Use stable deterministic values based on index to avoid random re-renders
      performance: p.isActive ? 85 + (idx * 2) % 15 : 40 + (idx * 4) % 30,
    })),
  [allProtocols, orchestrator.activeCoreId]);

  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Network className="w-5 h-5 text-indigo-400" />پروتکل‌ها — {toPersianNum(allProtocols.length)} پروتکل پشتیبانی‌شده</CardTitle>
          <CardDescription className="text-slate-500 text-xs">ترتیب بر اساس اولویت ضد DPI — هوش مصنوعی بهترین را انتخاب می‌کند</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {allProtocols.map(proto => (
              <div key={proto.key} className={`rounded-lg border p-3 transition-all ${proto.isActive ? 'border-emerald-500/30 bg-emerald-500/5' : 'border-slate-700/30 bg-slate-800/30'}`}>
                <div className="flex items-center justify-between mb-1.5">
                  <div className="flex items-center gap-2"><div className="w-3 h-3 rounded-full" style={{ backgroundColor: proto.color }} /><span className="text-slate-200 font-medium text-sm">{proto.nameFa}</span></div>
                  <div className="flex items-center gap-1">{proto.isActive && <Badge className="bg-emerald-500/20 text-emerald-400 border-emerald-500/30 text-[10px]">فعال</Badge>}<Badge variant="outline" className="text-[9px] text-slate-500 border-slate-600/50">اولویت {toPersianNum(proto.priority)}</Badge></div>
                </div>
                <p className="text-[10px] text-slate-500 mb-1">{proto.name}</p>
                <p className="text-[10px] text-cyan-400/70 mb-1.5">ضد DPI: {proto.antiDpiMethodFa}</p>
                <div className="flex gap-1 flex-wrap">{proto.supportedCores.map(core => <span key={core.id} className="text-lg" title={core.nameFa}>{core.icon}</span>)}</div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Radio className="w-5 h-5 text-pink-400" />توزیع پروتکل‌ها</CardTitle></CardHeader>
        <CardContent>
          <div className="h-64 flex items-center justify-center">
            <ResponsiveContainer width="100%" height="100%">
              <RadarChart data={radarData}>
                <PolarGrid stroke="#334155" /><PolarAngleAxis dataKey="protocol" tick={{ fill: '#94a3b8', fontSize: 10 }} /><PolarRadiusAxis tick={{ fill: '#64748b', fontSize: 9 }} />
                <Radar name="هسته‌ها" dataKey="cores" stroke="#6366f1" fill="#6366f1" fillOpacity={0.2} />
                <Radar name="عملکرد" dataKey="performance" stroke="#10b981" fill="#10b981" fillOpacity={0.2} />
                <Legend wrapperStyle={{ color: '#94a3b8', fontSize: 12 }} />
              </RadarChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// Traffic Routing Panel
// ──────────────────────────────────────────────
function TrafficRoutingPanel() {
  const { routing, setRoutingMode, toggleIranBypass, setDnsMode, setDnsProvider, connected } = useUnifiedShieldStore();

  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><GitBranch className="w-5 h-5 text-orange-400" />حالت مسیریابی</CardTitle></CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-3 mb-4">
            {[{ mode: 'full-vpn' as const, label: 'VPN کامل', desc: 'تمام ترافیک از تونل', icon: Shield }, { mode: 'split-tunnel' as const, label: 'تونل تقسیم', desc: 'ایران مستقیم، بقیه تونل', icon: ShieldHalf }, { mode: 'selective' as const, label: 'انتخابی', desc: 'فقط اپ‌های انتخابی', icon: MonitorSmartphone }].map(item => (
              <button key={item.mode} onClick={() => setRoutingMode(item.mode)} className={`rounded-lg border p-3 text-center transition-all ${routing.mode === item.mode ? 'border-orange-500/50 bg-orange-500/10' : 'border-slate-700/30 bg-slate-800/30 hover:bg-slate-800/50'}`}>
                <item.icon className={`w-6 h-6 mx-auto mb-1 ${routing.mode === item.mode ? 'text-orange-400' : 'text-slate-500'}`} />
                <p className={`text-sm font-medium ${routing.mode === item.mode ? 'text-orange-300' : 'text-slate-400'}`}>{item.label}</p>
                <p className="text-[10px] text-slate-500 mt-1">{item.desc}</p>
              </button>
            ))}
          </div>
          <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3 mb-4"><div className="flex items-center gap-2"><Globe className="w-5 h-5 text-emerald-400" /><div><p className="text-sm text-slate-200">bypass آی‌پی‌های ایران</p><p className="text-xs text-slate-500">حذف خودکار ترافیک داخلی</p></div></div><Switch checked={routing.iranIpBypass} onCheckedChange={toggleIranBypass} /></div>
          <div className="grid grid-cols-2 gap-3">
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3"><div className="flex items-center gap-2"><Network className="w-4 h-4 text-violet-400" /><span className="text-sm text-slate-300">IPv6</span></div><Switch checked={routing.ipv6Enabled} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ routing: { ...s.routing, ipv6Enabled: v } }))} /></div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3"><div className="flex items-center gap-2"><Radio className="w-4 h-4 text-cyan-400" /><span className="text-sm text-slate-300">P2P/تورنت</span></div><Switch checked={routing.p2pRouting} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ routing: { ...s.routing, p2pRouting: v } }))} /></div>
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Search className="w-5 h-5 text-blue-400" />تنظیمات DNS</CardTitle></CardHeader>
        <CardContent>
          <div className="flex gap-2 mb-4">
            {[{ mode: 'doh' as const, label: 'DoH', desc: 'DNS over HTTPS' }, { mode: 'dot' as const, label: 'DoT', desc: 'DNS over TLS' }, { mode: 'plain' as const, label: 'ساده', desc: 'بدون رمزگذاری' }].map(item => (
              <button key={item.mode} onClick={() => setDnsMode(item.mode)} className={`flex-1 rounded-lg border p-2 text-center transition-all ${routing.dnsMode === item.mode ? 'border-blue-500/50 bg-blue-500/10' : 'border-slate-700/30 bg-slate-800/30'}`}>
                <p className={`text-sm font-medium ${routing.dnsMode === item.mode ? 'text-blue-300' : 'text-slate-400'}`}>{item.label}</p>
                <p className="text-[10px] text-slate-500">{item.desc}</p>
              </button>
            ))}
          </div>
          <div className="space-y-2">{DNS_PROVIDERS.map(provider => (
            <button key={provider.id} onClick={() => setDnsProvider(provider.id)} className={`w-full flex items-center justify-between rounded-lg border p-2.5 transition-all ${routing.activeDnsProvider === provider.id ? 'border-blue-500/50 bg-blue-500/10' : 'border-slate-700/30 bg-slate-800/30'}`}>
              <div className="flex items-center gap-2"><Lock className="w-4 h-4 text-slate-400" /><div><p className={`text-sm ${routing.activeDnsProvider === provider.id ? 'text-blue-300' : 'text-slate-300'}`}>{provider.nameFa}</p><p className="text-[10px] text-slate-500">{provider.url}</p></div></div>
              {routing.activeDnsProvider === provider.id && <CheckCircle2 className="w-4 h-4 text-blue-400" />}
            </button>
          ))}</div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Layers className="w-5 h-5 text-teal-400" />قوانین تونل تقسیم</CardTitle></CardHeader>
        <CardContent><div className="space-y-2 max-h-72 overflow-y-auto custom-scrollbar">{routing.splitRules.map(rule => (
          <div key={rule.id} className="flex items-center justify-between bg-slate-700/30 rounded-lg p-2.5">
            <div className="flex items-center gap-2"><Switch checked={rule.enabled} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ routing: { ...s.routing, splitRules: s.routing.splitRules.map(r => r.id === rule.id ? { ...r, enabled: v } : r) } }))} className="scale-75" /><span className="text-sm text-slate-300">{rule.appFa}</span></div>
            <Badge variant="outline" className={`text-[10px] ${rule.route === 'vpn' ? 'border-emerald-500/30 text-emerald-400' : 'border-amber-500/30 text-amber-400'}`}>{rule.route === 'vpn' ? 'تونل' : 'مستقیم'}</Badge>
          </div>
        ))}</div></CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// OTA Update Panel
// ──────────────────────────────────────────────
function OTAUpdatePanel() {
  const { ota } = useUnifiedShieldStore();
  const typeLabels: Record<string, { label: string; icon: React.ReactNode; color: string }> = {
    'core-binary': { label: 'باینری هسته', icon: <Cpu className="w-4 h-4" />, color: 'text-violet-400' },
    'block-db': { label: 'پایگاه مسدودیت', icon: <Database className="w-4 h-4" />, color: 'text-red-400' },
    'ai-weights': { label: 'وزن‌های AI', icon: <Brain className="w-4 h-4" />, color: 'text-pink-400' },
    'node-list': { label: 'لیست نودها', icon: <Server className="w-4 h-4" />, color: 'text-cyan-400' },
  };

  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><CloudDownload className="w-5 h-5 text-sky-400" />به‌روزرسانی OTA (GitHub Releases API)</CardTitle><CardDescription className="text-slate-500 text-xs">بررسی خودکار هر ۶ ساعت — وصله دلتا — تأیید SHA256</CardDescription></CardHeader>
        <CardContent>
          <div className="flex items-center justify-between mb-4 text-xs"><span className="text-slate-500">آخرین بررسی: {new Date(ota.lastCheck).toLocaleString('fa-IR')}</span><span className="text-slate-500">بررسی بعدی: {new Date(ota.nextCheck).toLocaleString('fa-IR')}</span></div>
          <div className="space-y-3">{ota.updates.map(update => { const typeInfo = typeLabels[update.type]; return (
            <div key={update.id} className={`rounded-lg border p-3 ${update.status === 'installed' ? 'border-emerald-500/20 bg-emerald-500/5' : update.status === 'available' ? 'border-sky-500/20 bg-sky-500/5' : 'border-slate-700/30 bg-slate-800/30'}`}>
              <div className="flex items-start justify-between"><div className="flex items-center gap-2"><span className={typeInfo?.color}>{typeInfo?.icon}</span><div><p className="text-sm text-slate-200">{typeInfo?.label}</p><p className="text-xs text-slate-500">{update.target}</p></div></div>
              <Badge variant="outline" className={`text-[10px] ${update.status === 'installed' ? 'border-emerald-500/30 text-emerald-400' : 'border-sky-500/30 text-sky-400'}`}>{update.status === 'installed' ? 'نصب‌شده' : 'در دسترس'}</Badge></div>
              <div className="flex items-center gap-3 mt-2 text-xs text-slate-500"><span>{update.currentVersion} ← {update.version}</span><span>{formatBytes(update.size)}</span>{update.deltaPatch && <Badge variant="secondary" className="text-[9px] bg-slate-700/50">دلتا</Badge>}
              {ota.sha256Verification && <div className="flex items-center gap-1 text-emerald-400/70"><FileCheck className="w-3 h-3" /><span>SHA256</span></div>}</div>
            </div>
          ); })}</div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Settings className="w-5 h-5 text-slate-400" />تنظیمات</CardTitle></CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3"><div><p className="text-sm text-slate-200">به‌روزرسانی خودکار</p><p className="text-xs text-slate-500">نصب خودکار از GitHub API</p></div><Switch checked={ota.autoUpdate} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ ota: { ...s.ota, autoUpdate: v } }))} /></div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3"><div><p className="text-sm text-slate-200">بازگشت خودکار</p><p className="text-xs text-slate-500">بازگشت بعد از ۳ خطا</p></div><Switch checked={ota.rollbackEnabled} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ ota: { ...s.ota, rollbackEnabled: v } }))} /></div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3"><div><p className="text-sm text-slate-200">تأیید SHA256</p><p className="text-xs text-slate-500">بررسی امضای باینری</p></div><Switch checked={ota.sha256Verification} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ ota: { ...s.ota, sha256Verification: v } }))} /></div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// DPI Test Panel
// ──────────────────────────────────────────────
function DPITestPanel() {
  const { dpiResults, runDPITest, cores } = useUnifiedShieldStore();
  const [testing, setTesting] = useState(false);

  const handleTest = useCallback(() => { setTesting(true); setTimeout(() => { runDPITest(); setTesting(false); }, 3000); }, [runDPITest]);
  const passedCount = dpiResults.filter(r => r.connected).length;

  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Scan className="w-5 h-5 text-amber-400" />آزمایش شبیه‌سازی DPI ایران</CardTitle></CardHeader>
        <CardContent>
          <Button onClick={handleTest} disabled={testing} className="w-full bg-amber-500/20 text-amber-400 hover:bg-amber-500/30 border border-amber-500/30">
            {testing ? <><motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}><RefreshCw className="w-4 h-4 ml-2" /></motion.div>در حال آزمایش...</> : <><TestTube className="w-4 h-4 ml-2" />اجرای آزمایش DPI</>}
          </Button>
          {dpiResults.length > 0 && (
            <div className="mt-4 space-y-3">
              <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3"><span className="text-sm text-slate-300">نتیجه</span><div className="flex items-center gap-2"><span className={`text-lg font-bold ${passedCount >= 3 ? 'text-emerald-400' : 'text-red-400'}`}>{toPersianNum(passedCount)}/{toPersianNum(dpiResults.length)}</span>{passedCount >= 3 ? <CheckCircle2 className="w-5 h-5 text-emerald-400" /> : <XCircle className="w-5 h-5 text-red-400" />}</div></div>
              {dpiResults.map(r => (
                <div key={r.coreId} className={`rounded-lg border p-3 ${r.connected ? r.bypassLevel === 'full' ? 'border-emerald-500/20 bg-emerald-500/5' : 'border-yellow-500/20 bg-yellow-500/5' : 'border-red-500/20 bg-red-500/5'}`}>
                  <div className="flex items-center justify-between mb-2"><div className="flex items-center gap-2"><span className="text-lg">{cores.find(c => c.id === r.coreId)?.icon}</span><span className="text-sm text-slate-200">{r.coreNameFa}</span></div>
                  <Badge variant="outline" className={`text-[10px] ${r.bypassLevel === 'full' ? 'border-emerald-500/30 text-emerald-400' : r.bypassLevel === 'partial' ? 'border-yellow-500/30 text-yellow-400' : 'border-red-500/30 text-red-400'}`}>{r.bypassLevel === 'full' ? 'عبور کامل' : r.bypassLevel === 'partial' ? 'جزئی' : 'مسدود'}</Badge></div>
                  <div className="flex items-center gap-3 text-xs text-slate-500">{r.connected && <span>تأخیر: {toPersianNum(r.latency)} ms</span>}<span>DPI: {r.dpiSignatureFa}</span></div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
      {/* DPI Signatures Reference */}
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Fingerprint className="w-5 h-5 text-red-400" />امضاهای DPI ایران</CardTitle></CardHeader>
        <CardContent>
          <div className="space-y-2">{IRAN_DPI_SIGNATURES.map((sig, i) => (
            <div key={i} className="flex items-center justify-between bg-slate-700/20 rounded-lg p-2.5 border border-slate-700/30">
              <div><p className="text-sm text-slate-300">{sig.signature}</p><p className="text-xs text-slate-500">{sig.descriptionFa}</p></div>
              <Badge variant="outline" className="text-[10px] border-red-500/30 text-red-400 font-mono">{sig.hex}</Badge>
            </div>
          ))}</div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// ISP Rules Panel
// ──────────────────────────────────────────────
function ISPRulesPanel() {
  const { orchestrator, cores } = useUnifiedShieldStore();

  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><MapPin className="w-5 h-5 text-amber-400" />قوانین ISP ایران</CardTitle><CardDescription className="text-slate-500 text-xs">هوش مصنوعی به صورت خودکار ISP را شناسایی و بهترین هسته را انتخاب می‌کند</CardDescription></CardHeader>
        <CardContent>
          <div className="space-y-3">{ISP_RULES.map(isp => (
            <div key={isp.id} className={`rounded-lg border p-4 transition-all ${orchestrator.ispRuleApplied === isp.id ? 'border-amber-500/40 bg-amber-500/5' : 'border-slate-700/30 bg-slate-800/30'}`}>
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2"><Globe className="w-5 h-5 text-amber-400" /><div><p className="text-slate-200 font-semibold">{isp.nameFa}</p><p className="text-xs text-slate-500">{isp.name}</p></div></div>
                {orchestrator.ispRuleApplied === isp.id && <Badge className="bg-amber-500/20 text-amber-400 border-amber-500/30 text-[10px]">فعال</Badge>}
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
                <div><span className="text-slate-400 block mb-1">هسته‌های ترجیحی:</span><div className="flex gap-1 flex-wrap">{isp.preferredCores.map(cId => { const c = cores.find(x => x.id === cId); return c ? <Badge key={cId} variant="outline" className="text-[10px] border-emerald-500/30 text-emerald-400">{c.nameFa}</Badge> : null; })}</div></div>
                <div><span className="text-slate-400 block mb-1">پنهان‌سازی بهترین:</span><div className="flex gap-1 flex-wrap">{isp.bestObfuscationFa.map(o => <Badge key={o} variant="outline" className="text-[10px] border-cyan-500/30 text-cyan-400">{o}</Badge>)}</div></div>
                <div><span className="text-slate-400 block mb-1">پروتکل‌های مسدود:</span><div className="flex gap-1 flex-wrap">{isp.blockedProtocolsFa.length > 0 ? isp.blockedProtocolsFa.map(p => <Badge key={p} variant="outline" className="text-[10px] border-red-500/30 text-red-400">{p}</Badge>) : <span className="text-slate-600">اطلاعات محدود</span>}</div></div>
              </div>
            </div>
          ))}</div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// Kill Switch Panel
// ──────────────────────────────────────────────
function KillSwitchPanel() {
  const { killSwitch, toggleKillSwitch, toggleNetworkLock, connected } = useUnifiedShieldStore();

  return (
    <Card className="bg-slate-800/50 border-slate-700/50">
      <CardHeader className="pb-2">
        <CardTitle className="text-slate-200 text-base flex items-center gap-2">
          <ShieldOff className="w-5 h-5 text-red-400" />
          سوئیچ کشت (Kill Switch)
        </CardTitle>
        <CardDescription className="text-slate-500 text-xs">جلوگیری از نشت ترافیک در صورت قطع VPN — مسدودسازی کامل اینترنت</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className={`rounded-xl border p-4 transition-all ${killSwitch.enabled ? 'border-emerald-500/30 bg-emerald-500/5' : 'border-red-500/30 bg-red-500/5'}`}>
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-3">
                <div className={`w-12 h-12 rounded-xl flex items-center justify-center ${killSwitch.enabled ? 'bg-emerald-500/10 border border-emerald-500/30' : 'bg-red-500/10 border border-red-500/30'}`}>
                  {killSwitch.enabled ? <ShieldCheck className="w-6 h-6 text-emerald-400" /> : <ShieldOff className="w-6 h-6 text-red-400" />}
                </div>
                <div>
                  <p className="text-slate-200 font-semibold">سوئیچ کشت</p>
                  <p className={`text-xs ${killSwitch.enabled ? 'text-emerald-400' : 'text-red-400'}`}>{killSwitch.enabled ? 'فعال — ترافیک محافظت‌شده' : 'غیرفعال — خطر نشت'}</p>
                </div>
              </div>
              <Switch checked={killSwitch.enabled} onCheckedChange={toggleKillSwitch} />
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div className="bg-slate-700/30 rounded-lg p-2">
                <span className="text-slate-400 block">وضعیت</span>
                <p className={killSwitch.enabled ? 'text-emerald-400 font-bold' : 'text-red-400 font-bold'}>{killSwitch.enabled ? 'محافظت فعال' : 'خطر نشت'}</p>
              </div>
              <div className="bg-slate-700/30 rounded-lg p-2">
                <span className="text-slate-400 block">اتصال</span>
                <p className={connected ? 'text-emerald-400 font-bold' : 'text-red-400 font-bold'}>{connected ? 'متصل' : 'قطع'}</p>
              </div>
            </div>
          </div>
          <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
            <div className="flex items-center gap-2">
              <Ban className="w-5 h-5 text-amber-400" />
              <div>
                <p className="text-sm text-slate-200">مسدودسازی کامل هنگام قطع</p>
                <p className="text-xs text-slate-500">تمام ترافیک اینترنت قطع شود</p>
              </div>
            </div>
            <Switch checked={killSwitch.blockAllOnDisconnect} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ killSwitch: { ...s.killSwitch, blockAllOnDisconnect: v } }))} />
          </div>
          <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
            <div className="flex items-center gap-2">
              <Lock className="w-5 h-5 text-cyan-400" />
              <div>
                <p className="text-sm text-slate-200">قفل شبکه (Network Lock)</p>
                <p className="text-xs text-slate-500">جلوگیری از هرگونه اتصال خارج از تونل</p>
              </div>
            </div>
            <Switch checked={killSwitch.networkLock} onCheckedChange={toggleNetworkLock} />
          </div>
          {killSwitch.allowedApps.length > 0 && (
            <div className="bg-slate-700/20 rounded-lg p-3">
              <p className="text-xs text-slate-400 mb-2">اپلیکیشن‌های مجاز (حین قطع):</p>
              <div className="flex gap-1 flex-wrap">
                {killSwitch.allowedApps.map(app => <Badge key={app} variant="outline" className="text-[10px] border-amber-500/30 text-amber-400">{app}</Badge>)}
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

// ──────────────────────────────────────────────
// Auto-Reconnect Panel
// ──────────────────────────────────────────────
function AutoReconnectPanel() {
  const { autoReconnect, attemptReconnect, resetAutoReconnect, connected, addLog } = useUnifiedShieldStore();
  const [reconnecting, setReconnecting] = useState(false);

  const handleReconnect = useCallback(() => {
    setReconnecting(true);
    addLog({ type: 'reconnect', message: 'Manual reconnect attempt initiated', messageFa: 'تلاش اتصال مجدد دستی آغاز شد' });
    attemptReconnect();
    setTimeout(() => {
      setReconnecting(false);
    }, 3000);
  }, [attemptReconnect, addLog]);

  const statusLabels: Record<string, { label: string; color: string }> = {
    idle: { label: 'آماده', color: 'text-slate-400' },
    reconnecting: { label: 'در حال اتصال مجدد', color: 'text-yellow-400' },
    failed: { label: 'اتصال ناموفق', color: 'text-red-400' },
    connected: { label: 'متصل', color: 'text-emerald-400' },
  };

  const currentStatus = statusLabels[autoReconnect.reconnectStatus] ?? statusLabels.idle;

  return (
    <Card className="bg-slate-800/50 border-slate-700/50">
      <CardHeader className="pb-2">
        <CardTitle className="text-slate-200 text-base flex items-center gap-2">
          <RefreshCw className="w-5 h-5 text-cyan-400" />
          اتصال مجدد خودکار
        </CardTitle>
        <CardDescription className="text-slate-500 text-xs">اتصال مجدد خودکار با Backoff نمایی در صورت قطعی</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className={`rounded-xl border p-4 transition-all ${autoReconnect.reconnectStatus === 'connected' ? 'border-emerald-500/30 bg-emerald-500/5' : autoReconnect.reconnectStatus === 'reconnecting' ? 'border-yellow-500/30 bg-yellow-500/5' : autoReconnect.reconnectStatus === 'failed' ? 'border-red-500/30 bg-red-500/5' : 'border-slate-700/30 bg-slate-800/30'}`}>
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-3">
                <div className={`w-12 h-12 rounded-xl flex items-center justify-center ${autoReconnect.reconnectStatus === 'connected' ? 'bg-emerald-500/10 border border-emerald-500/30' : autoReconnect.reconnectStatus === 'reconnecting' ? 'bg-yellow-500/10 border border-yellow-500/30' : 'bg-slate-700/30'}`}>
                  {autoReconnect.reconnectStatus === 'reconnecting' ? <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}><RefreshCw className="w-6 h-6 text-yellow-400" /></motion.div> : autoReconnect.reconnectStatus === 'connected' ? <CheckCircle2 className="w-6 h-6 text-emerald-400" /> : autoReconnect.reconnectStatus === 'failed' ? <XCircle className="w-6 h-6 text-red-400" /> : <WifiOff className="w-6 h-6 text-slate-400" />}
                </div>
                <div>
                  <p className="text-slate-200 font-semibold">وضعیت اتصال مجدد</p>
                  <p className={`text-xs ${currentStatus.color}`}>{currentStatus.label}</p>
                </div>
              </div>
              <Switch checked={autoReconnect.enabled} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ autoReconnect: { ...s.autoReconnect, enabled: v } }))} />
            </div>
            <div className="grid grid-cols-3 gap-2 text-xs">
              <div className="bg-slate-700/30 rounded-lg p-2 text-center">
                <span className="text-slate-400 block">تلاش فعلی</span>
                <p className="text-cyan-400 font-bold">{toPersianNum(autoReconnect.retryCount)}</p>
              </div>
              <div className="bg-slate-700/30 rounded-lg p-2 text-center">
                <span className="text-slate-400 block">حداکثر تلاش</span>
                <p className="text-violet-400 font-bold">{toPersianNum(autoReconnect.maxRetries)}</p>
              </div>
              <div className="bg-slate-700/30 rounded-lg p-2 text-center">
                <span className="text-slate-400 block">فاصله (ms)</span>
                <p className="text-amber-400 font-bold">{toPersianNum(autoReconnect.retryInterval)}</p>
              </div>
            </div>
          </div>
          <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-5 h-5 text-violet-400" />
              <div>
                <p className="text-sm text-slate-200">Backoff نمایی</p>
                <p className="text-xs text-slate-500">افزایش تدریجی فاصله بین تلاش‌ها</p>
              </div>
            </div>
            <Switch checked={autoReconnect.exponentialBackoff} onCheckedChange={(v) => useUnifiedShieldStore.setState(s => ({ autoReconnect: { ...s.autoReconnect, exponentialBackoff: v } }))} />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="bg-slate-700/30 rounded-lg p-3">
              <label className="text-xs text-slate-400 block mb-1">حداکثر تلاش</label>
              <input type="number" value={autoReconnect.maxRetries} min={1} max={50} onChange={(e) => useUnifiedShieldStore.setState(s => ({ autoReconnect: { ...s.autoReconnect, maxRetries: parseInt(e.target.value) || 10 } }))} className="w-full bg-slate-800 border border-slate-600/50 rounded px-2 py-1 text-sm text-slate-200 focus:outline-none focus:border-cyan-500/50" />
            </div>
            <div className="bg-slate-700/30 rounded-lg p-3">
              <label className="text-xs text-slate-400 block mb-1">فاصله اولیه (ms)</label>
              <input type="number" value={autoReconnect.retryInterval} min={500} max={30000} step={500} onChange={(e) => useUnifiedShieldStore.setState(s => ({ autoReconnect: { ...s.autoReconnect, retryInterval: parseInt(e.target.value) || 3000 } }))} className="w-full bg-slate-800 border border-slate-600/50 rounded px-2 py-1 text-sm text-slate-200 focus:outline-none focus:border-cyan-500/50" />
            </div>
          </div>
          {!connected && (
            <Button onClick={handleReconnect} disabled={reconnecting || !autoReconnect.enabled} className="w-full bg-cyan-500/20 text-cyan-400 hover:bg-cyan-500/30 border border-cyan-500/30">
              {reconnecting ? <><motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}><RefreshCw className="w-4 h-4 ml-2" /></motion.div>در حال اتصال...</> : <><RefreshCw className="w-4 h-4 ml-2" />اتصال مجدد دستی</>}
            </Button>
          )}
          {autoReconnect.reconnectStatus === 'failed' && (
            <Button onClick={resetAutoReconnect} variant="ghost" className="w-full text-xs text-amber-400 hover:text-amber-300 hover:bg-amber-500/10 border border-amber-500/20">
              <RotateCcw className="w-3 h-3 ml-1" />بازنشانی تلاش‌های اتصال مجدد
            </Button>
          )}
          {autoReconnect.lastReconnectAttempt > 0 && (
            <div className="text-xs text-slate-500 text-center">آخرین تلاش: {new Date(autoReconnect.lastReconnectAttempt).toLocaleString('fa-IR')}</div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

// ──────────────────────────────────────────────
// Connection Logs Panel
// ──────────────────────────────────────────────
function ConnectionLogsPanel() {
  const { connectionLogs, addLog, cores } = useUnifiedShieldStore();
  const [filterType, setFilterType] = useState<ConnectionLogEntry['type'] | 'all'>('all');
  const logTypes: Array<{ value: ConnectionLogEntry['type'] | 'all'; label: string }> = [
    { value: 'all', label: 'همه' },
    { value: 'connect', label: 'اتصال' },
    { value: 'disconnect', label: 'قطع' },
    { value: 'switch', label: 'تعویض' },
    { value: 'block', label: 'مسدودیت' },
    { value: 'reconnect', label: 'اتصال مجدد' },
    { value: 'dpi-detect', label: 'DPI' },
    { value: 'update', label: 'به‌روزرسانی' },
    { value: 'error', label: 'خطا' },
  ];

  const filteredLogs = filterType === 'all' ? connectionLogs : connectionLogs.filter(l => l.type === filterType);

  return (
    <Card className="bg-slate-800/50 border-slate-700/50">
      <CardHeader className="pb-2">
        <CardTitle className="text-slate-200 text-base flex items-center gap-2">
          <FileText className="w-5 h-5 text-teal-400" />
          لاگ‌های اتصال — {toPersianNum(connectionLogs.length)} رکورد
        </CardTitle>
        <CardDescription className="text-slate-500 text-xs">تاریخچه کامل اتصال، تعویض، شناسایی DPI و خطاها</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="flex gap-1.5 flex-wrap">
            {logTypes.map(t => (
              <button key={t.value} onClick={() => setFilterType(t.value)} className={`rounded-lg border px-2.5 py-1 text-xs transition-all ${filterType === t.value ? 'border-teal-500/50 bg-teal-500/10 text-teal-300' : 'border-slate-700/30 bg-slate-800/30 text-slate-400 hover:bg-slate-800/50'}`}>
                {t.label}
              </button>
            ))}
          </div>
          <div className="space-y-2 max-h-96 overflow-y-auto custom-scrollbar">
            <AnimatePresence>
              {filteredLogs.map(log => (
                <motion.div key={log.id} initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0 }} className={`rounded-lg border p-3 ${getLogTypeBg(log.type)}`}>
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className={`text-[9px] ${getLogTypeBg(log.type)} ${getLogTypeColor(log.type)}`}>{getLogTypeLabel(log.type)}</Badge>
                      <span className="text-sm text-slate-200">{log.messageFa}</span>
                    </div>
                    <span className="text-[10px] text-slate-500 whitespace-nowrap">{new Date(log.timestamp).toLocaleTimeString('fa-IR')}</span>
                  </div>
                  {log.coreId && (
                    <div className="flex items-center gap-1 mt-1">
                      {(() => { const core = cores.find(c => c.id === log.coreId); return core ? <><span className="text-sm">{core.icon}</span><span className="text-[10px] text-slate-500">{core.nameFa}</span></> : null; })()}
                    </div>
                  )}
                  {log.details && Object.keys(log.details).length > 0 && (
                    <div className="flex gap-1 mt-1 flex-wrap">
                      {Object.entries(log.details).map(([k, v]) => <Badge key={k} variant="secondary" className="text-[9px] bg-slate-700/50 text-slate-400">{k}: {v}</Badge>)}
                    </div>
                  )}
                </motion.div>
              ))}
            </AnimatePresence>
            {filteredLogs.length === 0 && <div className="text-center text-slate-500 py-8"><FileText className="w-8 h-8 mx-auto mb-2 opacity-50" /><p className="text-sm">لاگی یافت نشد</p></div>}
          </div>
          <div className="flex gap-2">
            <Button onClick={() => addLog({ type: 'connect', message: 'Manual log test', messageFa: 'تست لاگ دستی', coreId: cores[0]?.id })} variant="ghost" className="text-xs text-teal-400 hover:text-teal-300 hover:bg-teal-500/10">
              <Plus className="w-3 h-3 ml-1" />افزودن لاگ تست
            </Button>
            <Button onClick={() => useUnifiedShieldStore.setState({ connectionLogs: [] })} variant="ghost" className="text-xs text-red-400 hover:text-red-300 hover:bg-red-500/10">
              <Trash2 className="w-3 h-3 ml-1" />پاک‌سازی
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// ──────────────────────────────────────────────
// Threat Intelligence Panel
// ──────────────────────────────────────────────
function ThreatIntelPanel() {
  const { threatIntel, cores } = useUnifiedShieldStore();

  const threatLevelProgress: Record<string, number> = { low: 20, medium: 50, high: 75, critical: 95 };
  const threatLevelColor: Record<string, string> = { low: 'text-emerald-400', medium: 'text-yellow-400', high: 'text-orange-400', critical: 'text-red-400' };
  const threatLevelBg: Record<string, string> = { low: 'bg-emerald-500/10 border-emerald-500/30', medium: 'bg-yellow-500/10 border-yellow-500/30', high: 'bg-orange-500/10 border-orange-500/30', critical: 'bg-red-500/10 border-red-500/30' };

  const severityData = [
    { name: 'بحرانی', value: threatIntel.activeThreats.filter(t => t.severity === 'critical').length, fill: '#ef4444' },
    { name: 'بالا', value: threatIntel.activeThreats.filter(t => t.severity === 'high').length, fill: '#f97316' },
    { name: 'متوسط', value: threatIntel.activeThreats.filter(t => t.severity === 'medium').length, fill: '#eab308' },
    { name: 'پایین', value: threatIntel.activeThreats.filter(t => t.severity === 'low').length, fill: '#10b981' },
  ].filter(d => d.value > 0);

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card className="bg-slate-800/50 border-slate-700/50">
          <CardHeader className="pb-2">
            <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Siren className="w-5 h-5 text-red-400" />سطح تهدید</CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`rounded-xl border p-4 ${threatLevelBg[threatIntel.threatLevel]}`}>
              <div className="flex items-center justify-between mb-3">
                <span className="text-sm text-slate-300">سطح فعلی</span>
                <span className={`text-xl font-bold ${threatLevelColor[threatIntel.threatLevel]}`}>{getSeverityLabel(threatIntel.threatLevel)}</span>
              </div>
              <Progress value={threatLevelProgress[threatIntel.threatLevel]} className="h-3 mb-3" />
              <div className="grid grid-cols-3 gap-2 text-xs">
                <div className="bg-slate-700/30 rounded-lg p-2 text-center"><span className="text-slate-400 block">تهدیدات فعال</span><p className="text-red-400 font-bold">{toPersianNum(threatIntel.activeThreats.length)}</p></div>
                <div className="bg-slate-700/30 rounded-lg p-2 text-center"><span className="text-slate-400 block">بی‌اثرشده</span><p className="text-emerald-400 font-bold">{toPersianNum(threatIntel.activeThreats.filter(t => t.mitigated).length)}</p></div>
                <div className="bg-slate-700/30 rounded-lg p-2 text-center"><span className="text-slate-400 block">دامنه‌های مسدود</span><p className="text-amber-400 font-bold">{toPersianNum(threatIntel.blockedDomainsCount)}</p></div>
              </div>
            </div>
            <div className="mt-3 text-xs text-slate-500">
              آخرین اسکن: {new Date(threatIntel.lastScan).toLocaleString('fa-IR')} — الگوهای DPI: {threatIntel.dpiPatternsUpdated}
            </div>
          </CardContent>
        </Card>
        <Card className="bg-slate-800/50 border-slate-700/50">
          <CardHeader className="pb-2">
            <CardTitle className="text-slate-200 text-base flex items-center gap-2"><BarChart3 className="w-5 h-5 text-amber-400" />توزیع شدت تهدیدات</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="h-48">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie data={severityData} cx="50%" cy="50%" innerRadius={40} outerRadius={70} paddingAngle={3} dataKey="value" nameKey="name">
                    {severityData.map((entry, index) => <Cell key={`cell-${index}`} fill={entry.fill} fillOpacity={0.7} />)}
                  </Pie>
                  <Tooltip contentStyle={{ background: '#1e293b', border: '1px solid #475569', borderRadius: 8 }} />
                  <Legend wrapperStyle={{ color: '#94a3b8', fontSize: 11 }} />
                </PieChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>
      </div>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><AlertOctagon className="w-5 h-5 text-orange-400" />تهدیدات فعال</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3 max-h-96 overflow-y-auto custom-scrollbar">
            {threatIntel.activeThreats.map(threat => (
              <div key={threat.id} className={`rounded-lg border p-3 transition-all ${threat.mitigated ? 'border-slate-700/30 bg-slate-800/30' : getSeverityBg(threat.severity)}`}>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className={`text-[10px] ${getSeverityBg(threat.severity)} ${getSeverityColor(threat.severity)}`}>{getSeverityLabel(threat.severity)}</Badge>
                    <span className="text-sm text-slate-200 font-medium">{threat.typeFa}</span>
                  </div>
                  {threat.mitigated ? <CheckCircle2 className="w-4 h-4 text-emerald-400" /> : <AlertTriangle className="w-4 h-4 text-red-400" />}
                </div>
                <p className="text-xs text-slate-400 mb-1.5">{threat.descriptionFa}</p>
                <div className="flex items-center justify-between text-xs">
                  <div className="flex items-center gap-1">
                    <ShieldCheck className="w-3 h-3 text-cyan-400" />
                    <span className="text-slate-500">اقطار متقابل:</span>
                    <span className="text-cyan-400">{threat.countermeasureFa}</span>
                  </div>
                  <span className="text-slate-600">{new Date(threat.detectedAt).toLocaleTimeString('fa-IR')}</span>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><ShieldHalf className="w-5 h-5 text-cyan-400" />اقطار متقابل فعال</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2 flex-wrap">
            {threatIntel.activeCountermeasures.map((cm, i) => (
              <Badge key={i} variant="outline" className="text-xs border-cyan-500/30 text-cyan-400 bg-cyan-500/5 px-3 py-1.5">{cm}</Badge>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// Advanced Settings Panel
// ──────────────────────────────────────────────
function AdvancedSettingsPanel() {
  const { advancedSettings } = useUnifiedShieldStore();
  const updateSettings = (partial: Partial<typeof advancedSettings>) => {
    useUnifiedShieldStore.setState(s => ({ advancedSettings: { ...s.advancedSettings, ...partial } }));
  };

  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Settings className="w-5 h-5 text-slate-400" />عمومی</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Globe className="w-5 h-5 text-amber-400" />
                <div>
                  <p className="text-sm text-slate-200">زبان</p>
                  <p className="text-xs text-slate-500">زبان رابط کاربری</p>
                </div>
              </div>
              <div className="flex gap-1">
                {[{ val: 'fa' as const, label: 'فارسی' }, { val: 'en' as const, label: 'English' }].map(l => (
                  <button key={l.val} onClick={() => updateSettings({ language: l.val })} className={`rounded-lg border px-3 py-1 text-xs transition-all ${advancedSettings.language === l.val ? 'border-amber-500/50 bg-amber-500/10 text-amber-300' : 'border-slate-700/30 bg-slate-800/30 text-slate-400'}`}>{l.label}</button>
                ))}
              </div>
            </div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Sun className="w-5 h-5 text-yellow-400" />
                <div>
                  <p className="text-sm text-slate-200">تم</p>
                  <p className="text-xs text-slate-500">ظاهر برنامه</p>
                </div>
              </div>
              <div className="flex gap-1">
                {[{ val: 'dark' as const, label: 'تاریک', icon: Moon }, { val: 'light' as const, label: 'روشن', icon: Sun }, { val: 'system' as const, label: 'سیستم', icon: Monitor }].map(t => (
                  <button key={t.val} onClick={() => updateSettings({ theme: t.val })} className={`rounded-lg border px-2 py-1 text-xs transition-all flex items-center gap-1 ${advancedSettings.theme === t.val ? 'border-yellow-500/50 bg-yellow-500/10 text-yellow-300' : 'border-slate-700/30 bg-slate-800/30 text-slate-400'}`}><t.icon className="w-3 h-3" />{t.label}</button>
                ))}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Cpu className="w-5 h-5 text-violet-400" />رفتار</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Power className="w-5 h-5 text-emerald-400" />
                <div>
                  <p className="text-sm text-slate-200">اجرا هنگام بوت</p>
                  <p className="text-xs text-slate-500">شروع خودکار با سیستم‌عامل</p>
                </div>
              </div>
              <Switch checked={advancedSettings.startOnBoot} onCheckedChange={(v) => updateSettings({ startOnBoot: v })} />
            </div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Zap className="w-5 h-5 text-cyan-400" />
                <div>
                  <p className="text-sm text-slate-200">اتصال خودکار</p>
                  <p className="text-xs text-slate-500">اتصال هنگام باز کردن برنامه</p>
                </div>
              </div>
              <Switch checked={advancedSettings.autoConnectOnLaunch} onCheckedChange={(v) => updateSettings({ autoConnectOnLaunch: v })} />
            </div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Activity className="w-5 h-5 text-pink-400" />
                <div>
                  <p className="text-sm text-slate-200">اعلان‌ها</p>
                  <p className="text-xs text-slate-500">اعلان تغییر وضعیت و هشدارها</p>
                </div>
              </div>
              <Switch checked={advancedSettings.notifications} onCheckedChange={(v) => updateSettings({ notifications: v })} />
            </div>
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Eye className="w-5 h-5 text-amber-400" />پنهان‌سازی و دیباگ</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Shield className="w-5 h-5 text-violet-400" />
                <div>
                  <p className="text-sm text-slate-200">حالت مخفی (Stealth Mode)</p>
                  <p className="text-xs text-slate-500">کاهش ردپای VPN — پنهان‌سازی پیشرفته</p>
                </div>
              </div>
              <Switch checked={advancedSettings.stealthMode} onCheckedChange={(v) => updateSettings({ stealthMode: v })} />
            </div>
            <div className="flex items-center justify-between bg-slate-700/30 rounded-lg p-3">
              <div className="flex items-center gap-2">
                <Bug className="w-5 h-5 text-red-400" />
                <div>
                  <p className="text-sm text-slate-200">حالت دیباگ</p>
                  <p className="text-xs text-slate-500">ثبت لاگ‌های دقیق برای عیب‌یابی</p>
                </div>
              </div>
              <Switch checked={advancedSettings.debugMode} onCheckedChange={(v) => updateSettings({ debugMode: v })} />
            </div>
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2">
          <CardTitle className="text-slate-200 text-base flex items-center gap-2"><Network className="w-5 h-5 text-teal-400" />شبکه</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-3">
            <div className="bg-slate-700/30 rounded-lg p-3">
              <label className="text-xs text-slate-400 block mb-1">مهلت اتصال (ms)</label>
              <input type="number" value={advancedSettings.connectionTimeout} min={1000} max={60000} step={1000} onChange={(e) => updateSettings({ connectionTimeout: parseInt(e.target.value) || 15000 })} className="w-full bg-slate-800 border border-slate-600/50 rounded px-2 py-1 text-sm text-slate-200 focus:outline-none focus:border-teal-500/50" />
            </div>
            <div className="bg-slate-700/30 rounded-lg p-3">
              <label className="text-xs text-slate-400 block mb-1">اندازه MTU</label>
              <input type="number" value={advancedSettings.mtuSize} min={576} max={9000} step={1} onChange={(e) => updateSettings({ mtuSize: parseInt(e.target.value) || 1500 })} className="w-full bg-slate-800 border border-slate-600/50 rounded px-2 py-1 text-sm text-slate-200 focus:outline-none focus:border-teal-500/50" />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// Architecture Panel
// ──────────────────────────────────────────────
function ArchitecturePanel() {
  return (
    <div className="space-y-4">
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Layers className="w-5 h-5 text-violet-400" />معماری سیستم MICAFP</CardTitle></CardHeader>
        <CardContent>
          <div className="space-y-3 font-mono text-xs">
            <div className="bg-violet-500/10 border border-violet-500/30 rounded-lg p-4 text-center">
              <p className="text-violet-300 font-bold mb-1">MICAFP Control Plane</p>
              <div className="grid grid-cols-3 gap-2 mt-2">
                <div className="bg-violet-500/10 rounded p-2 border border-violet-500/20"><Brain className="w-4 h-4 mx-auto text-violet-400 mb-1" /><p className="text-violet-300">AI Engine</p><p className="text-slate-500 text-[10px]">UCB Bandit</p></div>
                <div className="bg-violet-500/10 rounded p-2 border border-violet-500/20"><Activity className="w-4 h-4 mx-auto text-violet-400 mb-1" /><p className="text-violet-300">Health Monitor</p><p className="text-slate-500 text-[10px]">Heartbeat 5s</p></div>
                <div className="bg-violet-500/10 rounded p-2 border border-violet-500/20"><ArrowRightLeft className="w-4 h-4 mx-auto text-violet-400 mb-1" /><p className="text-violet-300">Core Switcher</p><p className="text-slate-500 text-[10px]">Auto-Rotate</p></div>
              </div>
            </div>
            <div className="flex justify-center"><div className="w-0.5 h-6 bg-slate-600" /></div>
            <div className="grid grid-cols-3 gap-2">
              <div className="bg-emerald-500/10 border border-emerald-500/30 rounded-lg p-3 text-center"><p className="text-emerald-300 font-bold">Pool 1</p><p className="text-slate-400">هیدیفای + سینگ‌باکس</p></div>
              <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-center"><p className="text-red-300 font-bold">Pool 2</p><p className="text-slate-400">مهساان‌جی + Xray GFW</p></div>
              <div className="bg-pink-500/10 border border-pink-500/30 rounded-lg p-3 text-center"><p className="text-pink-300 font-bold">Pool 3</p><p className="text-slate-400">آمنزیا + دیفیکس</p></div>
            </div>
            <div className="flex justify-center"><div className="w-0.5 h-6 bg-slate-600" /></div>
            <div className="bg-cyan-500/10 border border-cyan-500/30 rounded-lg p-3 text-center"><p className="text-cyan-300 font-bold">Psiphon Fallback Layer</p><p className="text-slate-400">GFW-knocker/psiphon-tunnel-core — بدون نیاز به سرور</p></div>
          </div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><MonitorSmartphone className="w-5 h-5 text-teal-400" />پلتفرم‌های پشتیبانی‌شده</CardTitle></CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">{PLATFORMS.map(p => (
            <div key={p.platform} className="rounded-lg border border-slate-700/30 bg-slate-800/30 p-3 text-center">
              <span className="text-2xl block mb-1">{p.icon}</span>
              <p className="text-slate-200 font-medium text-sm">{p.nameFa}</p>
              <p className="text-[10px] text-slate-500 mt-1">{p.tunnelTypeFa}</p>
              <p className="text-[10px] text-slate-600">{p.minVersion}</p>
            </div>
          ))}</div>
        </CardContent>
      </Card>
      <Card className="bg-slate-800/50 border-slate-700/50">
        <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Shield className="w-5 h-5 text-emerald-400" />ویژگی‌های امنیتی</CardTitle></CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-2 text-xs">
            {[
              { icon: Lock, label: 'رمزگذاری AES-256-GCM', color: 'text-emerald-400' },
              { icon: ShieldCheck, label: 'تأیید SHA256 باینری‌ها', color: 'text-cyan-400' },
              { icon: Eye, label: 'محافظت نشت DNS', color: 'text-violet-400' },
              { icon: Fingerprint, label: 'شناسایی امضای DPI', color: 'text-amber-400' },
              { icon: RotateCcw, label: 'بازگشت خودکار نسخه', color: 'text-pink-400' },
              { icon: Brain, label: 'AI محلی بدون ابر', color: 'text-indigo-400' },
            ].map((item, i) => (
              <div key={i} className="flex items-center gap-2 bg-slate-700/20 rounded p-2"><item.icon className={`w-4 h-4 ${item.color}`} /><span className="text-slate-300">{item.label}</span></div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ──────────────────────────────────────────────
// Stats Overview
// ──────────────────────────────────────────────
function StatsOverview() {
  const { stats, connected } = useUnifiedShieldStore();
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
      {[{ icon: Clock, label: 'زمان اتصال', value: connected ? formatUptime(stats.totalUptime) : '۰', color: 'text-emerald-400', bg: 'bg-emerald-500/10' },
        { icon: ArrowDownCircle, label: 'داده انتقالی', value: `${toPersianNum(stats.totalDataTransferred.down)} GB`, color: 'text-cyan-400', bg: 'bg-cyan-500/10' },
        { icon: ArrowRightLeft, label: 'تعویض هسته', value: toPersianNum(stats.switchesPerformed), color: 'text-violet-400', bg: 'bg-violet-500/10' },
        { icon: ShieldCheck, label: 'مسدودیت دفع‌شده', value: toPersianNum(stats.blockEventsAvoided), color: 'text-amber-400', bg: 'bg-amber-500/10' },
      ].map((stat, i) => (
        <div key={i} className={`${stat.bg} rounded-xl border border-slate-700/30 p-4`}>
          <stat.icon className={`w-5 h-5 ${stat.color} mb-2`} /><p className="text-xs text-slate-500">{stat.label}</p><p className={`text-lg font-bold ${stat.color}`}>{stat.value}</p>
        </div>
      ))}
    </div>
  );
}

// ──────────────────────────────────────────────
// Server Location
// ──────────────────────────────────────────────
function ServerLocationPanel() {
  const [selectedCountry, setSelectedCountry] = useState('DE');
  return (
    <Card className="bg-slate-800/50 border-slate-700/50">
      <CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Globe className="w-5 h-5 text-teal-400" />سرورها — {toPersianNum(COUNTRY_SERVERS.length)} کشور</CardTitle><CardDescription className="text-slate-500 text-xs">انتخاب خودکار توسط AI</CardDescription></CardHeader>
      <CardContent><div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2 max-h-64 overflow-y-auto custom-scrollbar">{COUNTRY_SERVERS.map(c => (
        <button key={c.code} onClick={() => setSelectedCountry(c.code)} className={`rounded-lg border p-2 text-center transition-all ${selectedCountry === c.code ? 'border-teal-500/50 bg-teal-500/10' : 'border-slate-700/30 bg-slate-800/30 hover:bg-slate-800/50'}`}>
          <p className={`text-sm ${selectedCountry === c.code ? 'text-teal-300' : 'text-slate-300'}`}>{c.nameFa}</p><p className="text-[10px] text-slate-500">{toPersianNum(c.servers)} سرور</p>
        </button>
      ))}</div></CardContent>
    </Card>
  );
}

// ──────────────────────────────────────────────
// Main Page
// ──────────────────────────────────────────────
export default function UnifiedShieldPage() {
  const { connected, updateCoreHealth, performAIOrchestration, cores, orchestrator } = useUnifiedShieldStore();
  const [activeTab, setActiveTab] = useState('dashboard');

  useEffect(() => {
    if (!connected) return;
    const interval = setInterval(() => { updateCoreHealth(); performAIOrchestration(); }, 15000);
    return () => clearInterval(interval);
  }, [connected, updateCoreHealth, performAIOrchestration]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950" dir="rtl">
      <header className="border-b border-slate-800/50 bg-slate-900/80 backdrop-blur-lg sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-cyan-500 flex items-center justify-center shadow-lg shadow-emerald-500/20"><ShieldCheck className="w-6 h-6 text-white" /></div>
            <div><h1 className="text-lg font-bold text-slate-100">یونیفایدشیلد — MICAFP</h1><p className="text-xs text-slate-500">موتور ضد سانسور هوشمند چند هسته‌ای — بهینه ایران</p></div>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className={`border-emerald-500/30 text-emerald-400 ${connected ? 'bg-emerald-500/10' : ''}`}>
              <motion.div className="w-2 h-2 rounded-full bg-emerald-400 ml-1" animate={connected ? { opacity: [1, 0.3, 1] } : {}} transition={{ duration: 2, repeat: Infinity }} />{connected ? 'متصل' : 'قطع'}
            </Badge>
            <Badge variant="outline" className="border-slate-600/50 text-slate-400">۹ هسته</Badge>
            <Badge variant="outline" className="border-slate-600/50 text-slate-400">۶ پلتفرم</Badge>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 py-6">
        <RealtimeStatusBar />
        <div className="flex flex-col items-center py-6 mb-4"><ConnectButton /><div className="mt-4"><PlatformSelector /></div></div>
        <div className="mb-6"><StatsOverview /></div>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
          <TabsList className="bg-slate-800/50 border border-slate-700/50 flex flex-wrap h-auto gap-1 p-1">
            <TabsTrigger value="dashboard" className="data-[state=active]:bg-emerald-500/20 data-[state=active]:text-emerald-400"><Activity className="w-4 h-4 ml-1" />داشبورد</TabsTrigger>
            <TabsTrigger value="ai" className="data-[state=active]:bg-violet-500/20 data-[state=active]:text-violet-400"><Brain className="w-4 h-4 ml-1" />هوش مصنوعی</TabsTrigger>
            <TabsTrigger value="protocols" className="data-[state=active]:bg-pink-500/20 data-[state=active]:text-pink-400"><Network className="w-4 h-4 ml-1" />پروتکل‌ها</TabsTrigger>
            <TabsTrigger value="routing" className="data-[state=active]:bg-orange-500/20 data-[state=active]:text-orange-400"><GitBranch className="w-4 h-4 ml-1" />مسیریابی</TabsTrigger>
            <TabsTrigger value="network" className="data-[state=active]:bg-cyan-500/20 data-[state=active]:text-cyan-400"><Gauge className="w-4 h-4 ml-1" />شبکه</TabsTrigger>
            <TabsTrigger value="geo" className="data-[state=active]:bg-indigo-500/20 data-[state=active]:text-indigo-400"><Globe className="w-4 h-4 ml-1" />سرورها</TabsTrigger>
            <TabsTrigger value="isp" className="data-[state=active]:bg-amber-500/20 data-[state=active]:text-amber-400"><MapPin className="w-4 h-4 ml-1" />ISP ایران</TabsTrigger>
            <TabsTrigger value="ota" className="data-[state=active]:bg-sky-500/20 data-[state=active]:text-sky-400"><CloudDownload className="w-4 h-4 ml-1" />به‌روزرسانی</TabsTrigger>
            <TabsTrigger value="dpi" className="data-[state=active]:bg-red-500/20 data-[state=active]:text-red-400"><Scan className="w-4 h-4 ml-1" />DPI</TabsTrigger>
            <TabsTrigger value="security" className="data-[state=active]:bg-rose-500/20 data-[state=active]:text-rose-400"><ShieldOff className="w-4 h-4 ml-1" />امنیت</TabsTrigger>
            <TabsTrigger value="audit" className="data-[state=active]:bg-fuchsia-500/20 data-[state=active]:text-fuchsia-400"><Fingerprint className="w-4 h-4 ml-1" />ممیزی</TabsTrigger>
            <TabsTrigger value="logs" className="data-[state=active]:bg-teal-500/20 data-[state=active]:text-teal-400"><FileText className="w-4 h-4 ml-1" />لاگ‌ها</TabsTrigger>
            <TabsTrigger value="threats" className="data-[state=active]:bg-orange-500/20 data-[state=active]:text-orange-400"><Siren className="w-4 h-4 ml-1" />تهدیدات</TabsTrigger>
            <TabsTrigger value="settings" className="data-[state=active]:bg-slate-500/20 data-[state=active]:text-slate-300"><Settings className="w-4 h-4 ml-1" />تنظیمات</TabsTrigger>
            <TabsTrigger value="arch" className="data-[state=active]:bg-teal-500/20 data-[state=active]:text-teal-400"><Layers className="w-4 h-4 ml-1" />معماری</TabsTrigger>
          </TabsList>

          <TabsContent value="dashboard">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
              <div className="lg:col-span-2 space-y-4">
                <Card className="bg-slate-800/50 border-slate-700/50"><CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Server className="w-5 h-5 text-emerald-400" />وضعیت ۹ هسته</CardTitle></CardHeader><CardContent><CoreGrid /></CardContent></Card>
                <ServerLocationPanel />
              </div>
              <div className="space-y-4">
                <SpeedMonitor />
                <Card className="bg-slate-800/50 border-slate-700/50"><CardHeader className="pb-2"><CardTitle className="text-slate-200 text-base flex items-center gap-2"><Cpu className="w-5 h-5 text-cyan-400" />هسته فعال</CardTitle></CardHeader><CardContent>
                  {connected && (() => { const ac = cores.find(c => c.id === orchestrator.activeCoreId); return ac ? (
                    <div className="space-y-2"><div className="flex items-center gap-2"><span className="text-2xl">{ac.icon}</span><div><p className="text-slate-200 font-bold">{ac.nameFa}</p><p className="text-xs text-slate-500">{ac.version}</p></div></div>
                    <div className="space-y-1 text-xs"><div className="flex justify-between"><span className="text-slate-400">تأخیر</span><span className="text-emerald-400">{toPersianNum(ac.health.latency)} ms</span></div><div className="flex justify-between"><span className="text-slate-400">دانلود</span><span className="text-cyan-400">{toPersianNum(ac.health.bandwidth.down)} Mbps</span></div><div className="flex justify-between"><span className="text-slate-400">DPI</span><span className={ac.health.dpiExposure > 30 ? 'text-red-400' : 'text-emerald-400'}>{toPersianNum(ac.health.dpiExposure)}</span></div></div></div>
                  ) : <p className="text-slate-500 text-sm">متصل نیست</p>; })() || <p className="text-slate-500 text-sm">ابتدا متصل شوید</p>}
                </CardContent></Card>
              </div>
            </div>
          </TabsContent>
          <TabsContent value="ai"><AIOrchestratorPanel /></TabsContent>
          <TabsContent value="protocols"><ProtocolStackPanel /></TabsContent>
          <TabsContent value="routing"><TrafficRoutingPanel /></TabsContent>
          <TabsContent value="network"><NetworkAnalyzerPanel /></TabsContent>
          <TabsContent value="geo"><GeoRouterPanel /></TabsContent>
          <TabsContent value="isp"><ISPRulesPanel /></TabsContent>
          <TabsContent value="ota"><OTAUpdatePanel /></TabsContent>
          <TabsContent value="dpi"><DPITestPanel /></TabsContent>
          <TabsContent value="security">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <KillSwitchPanel />
              <AutoReconnectPanel />
            </div>
          </TabsContent>
          <TabsContent value="audit"><SecurityAuditPanel /></TabsContent>
          <TabsContent value="logs"><ConnectionLogsPanel /></TabsContent>
          <TabsContent value="threats"><ThreatIntelPanel /></TabsContent>
          <TabsContent value="settings"><AdvancedSettingsPanel /></TabsContent>
          <TabsContent value="arch"><ArchitecturePanel /></TabsContent>
        </Tabs>
      </main>

      <footer className="border-t border-slate-800/50 bg-slate-900/50 mt-auto">
        <div className="max-w-7xl mx-auto px-4 py-4 flex flex-wrap items-center justify-between text-xs text-slate-600">
          <span>یونیفایدشیلد — MICAFP — موتور ضد سانسور هوشمند چند هسته‌ای (بهینه ایران)</span>
          <div className="flex items-center gap-3"><span>بدون روت</span><span>•</span><span>بدون سرور</span><span>•</span><span>تمام خودکار</span><span>•</span><span>۶ پلتفرم</span></div>
        </div>
      </footer>
    </div>
  );
}
