'use client';

import React, { useEffect, useState, useMemo, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Shield, ShieldCheck, ShieldAlert, Activity, Zap, Wifi, WifiOff,
  ArrowUpCircle, ArrowDownCircle, Timer, AlertTriangle, CheckCircle2,
  Eye, Lock, Server, Signal, Globe, RefreshCw,
  Radio, Layers, AlertOctagon,
} from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { useUnifiedShieldStore } from '@/lib/unified-shield-store';

function toPersianNum(n: number | string): string {
  const persianDigits = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];
  return String(n).replace(/\d/g, d => persianDigits[parseInt(d)]);
}

// ──────────────────────────────────────────────
// Animated Number
// ──────────────────────────────────────────────
function AnimatedNumber({ value, suffix = '' }: { value: number; suffix?: string }) {
  return (
    <motion.span
      key={value}
      initial={{ y: -10, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ type: 'spring', stiffness: 500, damping: 30 }}
      className="font-bold tabular-nums"
    >
      {toPersianNum(value)}{suffix}
    </motion.span>
  );
}

// ──────────────────────────────────────────────
// Connection Duration Timer
// ──────────────────────────────────────────────
function ConnectionTimer({ connected }: { connected: boolean }) {
  const [elapsed, setElapsed] = useState(0);
  const connectStartRef = useRef<number>(Date.now());

  useEffect(() => {
    if (!connected) {
      setElapsed(0);
      return;
    }
    connectStartRef.current = Date.now();
    const interval = setInterval(() => {
      setElapsed(Math.floor((Date.now() - connectStartRef.current) / 1000));
    }, 1000);
    return () => clearInterval(interval);
  }, [connected]);

  const hours = Math.floor(elapsed / 3600);
  const minutes = Math.floor((elapsed % 3600) / 60);
  const seconds = elapsed % 60;

  const formatNum = (n: number) => String(n).padStart(2, '0');

  return (
    <div className="flex items-center gap-0.5 font-mono text-sm">
      <AnimatedNumber value={parseInt(formatNum(hours))} />
      <span className="text-slate-600 mx-0.5">:</span>
      <AnimatedNumber value={parseInt(formatNum(minutes))} />
      <span className="text-slate-600 mx-0.5">:</span>
      <AnimatedNumber value={parseInt(formatNum(seconds))} />
    </div>
  );
}

// ──────────────────────────────────────────────
// VPN Status Indicator
// ──────────────────────────────────────────────
function VPNStatusIndicator() {
  const { connected, cores, orchestrator } = useUnifiedShieldStore();
  const activeCore = cores.find(c => c.id === orchestrator.activeCoreId);

  // Determine status
  const status = !connected ? 'disconnected' : activeCore ? 'connected' : 'connecting';

  const statusConfig = {
    connected: {
      color: 'text-emerald-400',
      bg: 'bg-emerald-500/20',
      border: 'border-emerald-500/30',
      glow: 'shadow-emerald-500/20',
      label: 'متصل',
      icon: ShieldCheck,
    },
    connecting: {
      color: 'text-yellow-400',
      bg: 'bg-yellow-500/20',
      border: 'border-yellow-500/30',
      glow: 'shadow-yellow-500/20',
      label: 'در حال اتصال',
      icon: RefreshCw,
    },
    disconnected: {
      color: 'text-slate-400',
      bg: 'bg-slate-500/10',
      border: 'border-slate-600/30',
      glow: '',
      label: 'قطع',
      icon: WifiOff,
    },
  };

  const cfg = statusConfig[status];
  const Icon = cfg.icon;

  return (
    <div className="flex items-center gap-2">
      <div className={`relative w-8 h-8 rounded-lg ${cfg.bg} border ${cfg.border} flex items-center justify-center shadow-lg ${cfg.glow}`}>
        {status === 'connecting' ? (
          <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
            <Icon className={`w-4 h-4 ${cfg.color}`} />
          </motion.div>
        ) : (
          <Icon className={`w-4 h-4 ${cfg.color}`} />
        )}
        {status === 'connected' && (
          <motion.div
            className="absolute inset-0 rounded-lg border border-emerald-400/30"
            animate={{ scale: [1, 1.2, 1], opacity: [0.5, 0, 0.5] }}
            transition={{ duration: 2, repeat: Infinity }}
          />
        )}
      </div>
      <div>
        <p className={`text-xs font-bold ${cfg.color}`}>{cfg.label}</p>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Active Core Display
// ──────────────────────────────────────────────
function ActiveCoreDisplay() {
  const { cores, orchestrator, connected } = useUnifiedShieldStore();
  const activeCore = cores.find(c => c.id === orchestrator.activeCoreId);

  if (!connected || !activeCore) {
    return (
      <div className="flex items-center gap-2">
        <div className="w-7 h-7 rounded-lg bg-slate-700/50 flex items-center justify-center">
          <Server className="w-3.5 h-3.5 text-slate-500" />
        </div>
        <span className="text-xs text-slate-500">هسته: —</span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        key={activeCore.id}
        className="w-7 h-7 rounded-lg flex items-center justify-center"
        style={{ backgroundColor: `${activeCore.color}15`, border: `1px solid ${activeCore.color}30` }}
      >
        <span className="text-sm">{activeCore.icon}</span>
      </motion.div>
      <div>
        <p className="text-xs text-slate-300 font-medium">{activeCore.nameFa}</p>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Speed Display
// ──────────────────────────────────────────────
function SpeedDisplay() {
  const { stats, connected } = useUnifiedShieldStore();

  return (
    <div className="flex items-center gap-4">
      <div className="flex items-center gap-1.5">
        <ArrowUpCircle className={`w-4 h-4 ${connected ? 'text-emerald-400' : 'text-slate-600'}`} />
        <div className="text-right">
          <p className="text-[9px] text-slate-500 leading-none">آپلود</p>
          <p className={`text-xs font-bold tabular-nums ${connected ? 'text-emerald-400' : 'text-slate-600'}`}>
            {connected ? <AnimatedNumber value={Math.round(stats.currentSpeed.up)} suffix=" Mb" /> : '—'}
          </p>
        </div>
      </div>
      <div className="w-px h-6 bg-slate-700/50" />
      <div className="flex items-center gap-1.5">
        <ArrowDownCircle className={`w-4 h-4 ${connected ? 'text-cyan-400' : 'text-slate-600'}`} />
        <div className="text-right">
          <p className="text-[9px] text-slate-500 leading-none">دانلود</p>
          <p className={`text-xs font-bold tabular-nums ${connected ? 'text-cyan-400' : 'text-slate-600'}`}>
            {connected ? <AnimatedNumber value={Math.round(stats.currentSpeed.down)} suffix=" Mb" /> : '—'}
          </p>
        </div>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// ISP Display
// ──────────────────────────────────────────────
function ISPDisplay() {
  const { orchestrator, connected } = useUnifiedShieldStore();

  return (
    <div className="flex items-center gap-2">
      <Globe className={`w-4 h-4 ${connected ? 'text-amber-400' : 'text-slate-600'}`} />
      <div className="text-right">
        <p className="text-[9px] text-slate-500 leading-none">ISP</p>
        <p className={`text-xs font-medium ${connected ? 'text-amber-300' : 'text-slate-600'}`}>
          {connected ? orchestrator.detectedISPFa : '—'}
        </p>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Security Status Indicator
// ──────────────────────────────────────────────
function SecurityStatusIndicator() {
  const { connected, killSwitch, cores, orchestrator } = useUnifiedShieldStore();
  const activeCore = cores.find(c => c.id === orchestrator.activeCoreId);

  const isSecure = connected && killSwitch.enabled && !activeCore?.health.dnsLeak;
  const isWarning = connected && (!killSwitch.enabled || activeCore?.health.dnsLeak);

  return (
    <div className="flex items-center gap-1.5">
      <motion.div
        className={`w-2.5 h-2.5 rounded-full ${
          !connected ? 'bg-slate-600' : isSecure ? 'bg-emerald-400' : isWarning ? 'bg-yellow-400' : 'bg-red-400'
        }`}
        animate={connected ? { scale: [1, 1.3, 1], opacity: [1, 0.6, 1] } : {}}
        transition={{ duration: 2, repeat: Infinity }}
      />
      <span className={`text-[10px] font-medium ${
        !connected ? 'text-slate-600' : isSecure ? 'text-emerald-400' : isWarning ? 'text-yellow-400' : 'text-red-400'
      }`}>
        {!connected ? 'ناامن' : isSecure ? 'ایمن' : 'خطر'}
      </span>
    </div>
  );
}

// ──────────────────────────────────────────────
// DPI Threat Level Indicator
// ──────────────────────────────────────────────
function DPIThreatIndicator() {
  const { threatIntel, connected } = useUnifiedShieldStore();

  const levelConfig = {
    low: { color: 'text-emerald-400', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30', label: 'کم' },
    medium: { color: 'text-yellow-400', bg: 'bg-yellow-500/10', border: 'border-yellow-500/30', label: 'متوسط' },
    high: { color: 'text-orange-400', bg: 'bg-orange-500/10', border: 'border-orange-500/30', label: 'بالا' },
    critical: { color: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/30', label: 'بحرانی' },
  };

  const cfg = connected ? levelConfig[threatIntel.threatLevel] : { color: 'text-slate-600', bg: 'bg-slate-500/10', border: 'border-slate-600/30', label: '—' };

  return (
    <div className="flex items-center gap-2">
      <Eye className={`w-4 h-4 ${cfg.color}`} />
      <div className="text-right">
        <p className="text-[9px] text-slate-500 leading-none">تهدید DPI</p>
        <div className={`inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] font-bold ${cfg.color} ${cfg.bg} border ${cfg.border}`}>
          {cfg.label}
        </div>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────
// Main Status Bar Component
// ──────────────────────────────────────────────
export default function RealtimeStatusBar() {
  const { connected } = useUnifiedShieldStore();

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      className="w-full"
    >
      <div
        className={`
          relative rounded-2xl border overflow-hidden
          ${connected
            ? 'border-emerald-500/20 bg-slate-900/80'
            : 'border-slate-700/30 bg-slate-900/60'
          }
          backdrop-blur-xl
        `}
        style={{
          boxShadow: connected
            ? '0 0 30px rgba(16,185,129,0.08), 0 4px 20px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.03)'
            : '0 4px 20px rgba(0,0,0,0.2), inset 0 1px 0 rgba(255,255,255,0.02)',
        }}
      >
        {/* Background gradient glow when connected */}
        {connected && (
          <div className="absolute inset-0 pointer-events-none">
            <div className="absolute inset-0 bg-gradient-to-r from-emerald-500/5 via-cyan-500/3 to-violet-500/5" />
            <motion.div
              className="absolute inset-0"
              style={{
                background: 'linear-gradient(90deg, transparent 0%, rgba(16,185,129,0.06) 50%, transparent 100%)',
              }}
              animate={{ x: ['-100%', '100%'] }}
              transition={{ duration: 4, repeat: Infinity, ease: 'linear' }}
            />
          </div>
        )}

        <div className="relative px-4 py-3">
          <div className="flex items-center justify-between gap-3 flex-wrap">
            {/* VPN Status + Active Core */}
            <div className="flex items-center gap-4">
              <VPNStatusIndicator />
              <ActiveCoreDisplay />
            </div>

            {/* Divider */}
            <div className="hidden md:block w-px h-8 bg-slate-700/50" />

            {/* Speed */}
            <SpeedDisplay />

            {/* Divider */}
            <div className="hidden md:block w-px h-8 bg-slate-700/50" />

            {/* ISP */}
            <ISPDisplay />

            {/* Divider */}
            <div className="hidden md:block w-px h-8 bg-slate-700/50" />

            {/* Connection Timer */}
            <div className="flex items-center gap-2">
              <Timer className={`w-4 h-4 ${connected ? 'text-violet-400' : 'text-slate-600'}`} />
              <div className="text-right">
                <p className="text-[9px] text-slate-500 leading-none">مدت اتصال</p>
                <ConnectionTimer connected={connected} />
              </div>
            </div>

            {/* Divider */}
            <div className="hidden md:block w-px h-8 bg-slate-700/50" />

            {/* Security + DPI */}
            <div className="flex items-center gap-4">
              <SecurityStatusIndicator />
              <DPIThreatIndicator />
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
