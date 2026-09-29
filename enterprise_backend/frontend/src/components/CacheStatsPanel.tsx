import React from 'react';
import { HealthCheckResponse } from '../types';
import { Cpu, Zap, Database, Activity, RefreshCw } from 'lucide-react';

interface CacheStatsPanelProps {
  health: HealthCheckResponse | null;
  onRefresh: () => void;
}

export const CacheStatsPanel: React.FC<CacheStatsPanelProps> = ({ health, onRefresh }) => {
  const l1Hits = health?.cache.l1_hits || 0;
  const l2Hits = health?.cache.l2_hits || 0;
  const misses = health?.cache.misses || 0;
  const total = l1Hits + l2Hits + misses;

  const l1Percent = total > 0 ? Math.round((l1Hits / total) * 100) : 0;
  const l2Percent = total > 0 ? Math.round((l2Hits / total) * 100) : 0;
  const missPercent = total > 0 ? Math.round((misses / total) * 100) : 0;

  return (
    <div className="glass-card rounded-2xl p-6 border border-slate-800 mb-8 relative overflow-hidden">
      {/* Background Accent */}
      <div className="absolute top-0 right-0 w-64 h-64 bg-indigo-500/5 rounded-full blur-3xl" />

      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <div className="p-2 rounded-lg bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
            <Activity className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-lg font-bold text-white">Multi-Tier L1 / L2 Cache Performance</h3>
            <p className="text-xs text-slate-400 font-mono">Live hit-ratio breakdown across Moka (L1) and Redis (L2)</p>
          </div>
        </div>
        <button
          onClick={onRefresh}
          className="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 text-slate-300 transition-all border border-slate-700 flex items-center gap-1.5 text-xs font-mono"
        >
          <RefreshCw className="w-3.5 h-3.5" /> Refresh Metrics
        </button>
      </div>

      {/* Metric Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">

        {/* L1 Moka Card */}
        <div className="p-4 rounded-xl bg-emerald-950/20 border border-emerald-500/30 flex items-center justify-between">
          <div>
            <div className="flex items-center gap-1.5 text-emerald-400 text-xs font-semibold font-mono mb-1">
              <Cpu className="w-4 h-4" /> L1 Moka In-Memory
            </div>
            <p className="text-2xl font-extrabold text-white font-mono">{l1Hits} <span className="text-xs text-slate-400 font-normal">hits</span></p>
            <p className="text-[11px] text-emerald-400/80 font-mono mt-0.5">&lt; 1 ms latency</p>
          </div>
          <div className="text-right">
            <span className="text-2xl font-bold text-emerald-400 font-mono">{l1Percent}%</span>
          </div>
        </div>

        {/* L2 Redis Card */}
        <div className="p-4 rounded-xl bg-cyan-950/20 border border-cyan-500/30 flex items-center justify-between">
          <div>
            <div className="flex items-center gap-1.5 text-cyan-400 text-xs font-semibold font-mono mb-1">
              <Zap className="w-4 h-4" /> L2 Redis Distributed
            </div>
            <p className="text-2xl font-extrabold text-white font-mono">{l2Hits} <span className="text-xs text-slate-400 font-normal">hits</span></p>
            <p className="text-[11px] text-cyan-400/80 font-mono mt-0.5">&lt; 5 ms latency</p>
          </div>
          <div className="text-right">
            <span className="text-2xl font-bold text-cyan-400 font-mono">{l2Percent}%</span>
          </div>
        </div>

        {/* Database Query Card */}
        <div className="p-4 rounded-xl bg-amber-950/20 border border-amber-500/30 flex items-center justify-between">
          <div>
            <div className="flex items-center gap-1.5 text-amber-400 text-xs font-semibold font-mono mb-1">
              <Database className="w-4 h-4" /> PostgreSQL SQLx Query
            </div>
            <p className="text-2xl font-extrabold text-white font-mono">{misses} <span className="text-xs text-slate-400 font-normal">queries</span></p>
            <p className="text-[11px] text-amber-400/80 font-mono mt-0.5">Disk / Index Query</p>
          </div>
          <div className="text-right">
            <span className="text-2xl font-bold text-amber-400 font-mono">{missPercent}%</span>
          </div>
        </div>
      </div>

      {/* Visual Proportion Bar */}
      <div className="w-full bg-slate-900 rounded-full h-3 flex overflow-hidden border border-slate-800">
        <div style={{ width: `${l1Percent}%` }} className="bg-emerald-500 transition-all duration-500" title={`L1 Moka: ${l1Percent}%`} />
        <div style={{ width: `${l2Percent}%` }} className="bg-cyan-500 transition-all duration-500" title={`L2 Redis: ${l2Percent}%`} />
        <div style={{ width: `${missPercent}%` }} className="bg-amber-500 transition-all duration-500" title={`PostgreSQL: ${missPercent}%`} />
      </div>
    </div>
  );
};
