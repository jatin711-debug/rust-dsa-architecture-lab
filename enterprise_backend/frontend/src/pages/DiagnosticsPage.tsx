import React from 'react';
import { HealthCheckResponse } from '../types';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { CacheStatsPanel } from '../components/CacheStatsPanel';
import { Cpu, Zap, Database, Server, Terminal, RefreshCw, Activity, Layers, ShieldCheck } from 'lucide-react';

interface DiagnosticsPageProps {
  health: HealthCheckResponse | null;
  onRefresh: () => void;
}

export const DiagnosticsPage: React.FC<DiagnosticsPageProps> = ({ health, onRefresh }) => {
  return (
    <div className="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8 animate-fade-in space-y-6">

      {/* Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-3xl font-black text-white">System & Cache Diagnostics</h2>
            <Badge variant="emerald">Live Telemetry</Badge>
          </div>
          <p className="text-xs text-slate-400 font-mono mt-1">
            Real-time diagnostic metrics from Axum 0.8, Tokio runtime, Moka L1, Redis 7 L2, and PostgreSQL
          </p>
        </div>

        <Button variant="outline" onClick={onRefresh} className="font-mono text-xs">
          <RefreshCw className="w-4 h-4" /> Refresh Telemetry
        </Button>
      </div>

      {/* Main Cache Visualizer Panel */}
      <CacheStatsPanel health={health} onRefresh={onRefresh} />

      {/* System Architecture Details Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">

        {/* Tier 1 */}
        <Card className="border-emerald-500/30">
          <CardHeader>
            <div className="p-2 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 w-fit mb-2">
              <Cpu className="w-6 h-6" />
            </div>
            <CardTitle className="text-xl font-bold text-emerald-300">L1 In-Memory Cache</CardTitle>
            <CardDescription className="font-mono">Moka Concurrent Cache (Rust)</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 font-mono text-xs text-slate-300">
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Latency Speed</p>
              <p className="text-emerald-400 font-bold text-sm">&lt; 1 Millisecond (&lt; 100µs)</p>
            </div>
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Capacity Strategy</p>
              <p className="text-slate-200">10,000 entries (TinyLFU eviction)</p>
            </div>
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Concurrency Model</p>
              <p className="text-slate-200">Non-blocking async lock-free reads</p>
            </div>
          </CardContent>
        </Card>

        {/* Tier 2 */}
        <Card className="border-cyan-500/30">
          <CardHeader>
            <div className="p-2 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 w-fit mb-2">
              <Zap className="w-6 h-6" />
            </div>
            <CardTitle className="text-xl font-bold text-cyan-300">L2 Distributed Cache</CardTitle>
            <CardDescription className="font-mono">Redis 7 Alpine Container</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 font-mono text-xs text-slate-300">
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Latency Speed</p>
              <p className="text-cyan-400 font-bold text-sm">&lt; 5 Milliseconds</p>
            </div>
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Driver Protocol</p>
              <p className="text-slate-200">Async ConnectionManager</p>
            </div>
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Invalidation Flow</p>
              <p className="text-slate-200">Invalidated on Product / Order updates</p>
            </div>
          </CardContent>
        </Card>

        {/* Persistence */}
        <Card className="border-amber-500/30">
          <CardHeader>
            <div className="p-2 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20 w-fit mb-2">
              <Database className="w-6 h-6" />
            </div>
            <CardTitle className="text-xl font-bold text-amber-300">PostgreSQL Database</CardTitle>
            <CardDescription className="font-mono">SQLx Compile-Checked Pool</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 font-mono text-xs text-slate-300">
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Connection Pool</p>
              <p className="text-amber-400 font-bold text-sm">20 Max Connections (5 Min)</p>
            </div>
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Transaction Model</p>
              <p className="text-slate-200">ACID Isolation + Row Lock (`FOR UPDATE`)</p>
            </div>
            <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800 space-y-1">
              <p className="text-slate-500 uppercase text-[10px]">Migrations</p>
              <p className="text-slate-200">Auto-applied on server startup</p>
            </div>
          </CardContent>
        </Card>

      </div>
    </div>
  );
};
