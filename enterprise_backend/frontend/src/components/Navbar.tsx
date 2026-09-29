import React from 'react';
import { User, HealthCheckResponse, CacheSourceHeader } from '../types';
import { ShoppingBag, Zap, LogOut, UserCheck, ShieldCheck, Layers, Cpu, Database, Activity, Package, Clock } from 'lucide-react';
import { Button } from './ui/button';
import { Badge } from './ui/badge';

export type PageView = 'catalog' | 'inventory' | 'orders' | 'diagnostics' | 'auth';

interface NavbarProps {
  activeView: PageView;
  setActiveView: (view: PageView) => void;
  user: User | null;
  health: HealthCheckResponse | null;
  cartCount: number;
  lastCacheSource: CacheSourceHeader | null;
  onOpenCart: () => void;
  onLogout: () => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  activeView,
  setActiveView,
  user,
  health,
  cartCount,
  lastCacheSource,
  onOpenCart,
  onLogout,
}) => {
  const isHealthy = health?.status === 'healthy';

  return (
    <header className="sticky top-0 z-40 w-full glass-panel border-b border-slate-800/80">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">

        {/* Brand Logo & Navigation Links */}
        <div className="flex items-center gap-6">
          <div
            onClick={() => setActiveView('catalog')}
            className="flex items-center gap-3 cursor-pointer group"
          >
            <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-indigo-600 via-purple-600 to-pink-500 flex items-center justify-center glow-indigo group-hover:scale-105 transition-transform">
              <Zap className="w-5 h-5 text-white animate-pulse" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-xl font-bold text-gradient">NEXUS CORE</span>
                <span className="text-[10px] uppercase font-mono px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                  Axum 0.8
                </span>
              </div>
            </div>
          </div>

          {/* Navigation Links */}
          <nav className="hidden md:flex items-center gap-1 bg-slate-950/60 p-1 rounded-xl border border-slate-800">
            <button
              onClick={() => setActiveView('catalog')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 ${
                activeView === 'catalog'
                  ? 'bg-indigo-600 text-white shadow-md'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <Package className="w-3.5 h-3.5" /> Catalog
            </button>

            {user?.role === 'StoreAdmin' && (
              <button
                onClick={() => setActiveView('inventory')}
                className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 ${
                  activeView === 'inventory'
                    ? 'bg-purple-600 text-white shadow-md'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <Layers className="w-3.5 h-3.5" /> Inventory
              </button>
            )}

            <button
              onClick={() => setActiveView('orders')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 ${
                activeView === 'orders'
                  ? 'bg-indigo-600 text-white shadow-md'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <Clock className="w-3.5 h-3.5" /> Orders
            </button>

            <button
              onClick={() => setActiveView('diagnostics')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 ${
                activeView === 'diagnostics'
                  ? 'bg-indigo-600 text-white shadow-md'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <Activity className="w-3.5 h-3.5" /> Telemetry
            </button>
          </nav>
        </div>

        {/* Live Diagnostics Badge */}
        <div className="hidden lg:flex items-center gap-3 px-3 py-1.5 rounded-xl bg-slate-950/80 border border-slate-800 text-xs font-mono">
          <div className="flex items-center gap-2">
            <span className={`w-2 h-2 rounded-full ${isHealthy ? 'bg-emerald-400 animate-ping' : 'bg-rose-500'}`} />
            <span className={isHealthy ? 'text-emerald-400 font-medium' : 'text-rose-400'}>
              {isHealthy ? 'Backend Live' : 'Connecting...'}
            </span>
          </div>

          {lastCacheSource && (
            <>
              <div className="h-3 w-px bg-slate-800" />
              {lastCacheSource === 'L1_HIT' && (
                <span className="text-emerald-400 font-bold flex items-center gap-1">
                  <Cpu className="w-3 h-3" /> L1 Hit (&lt;1ms)
                </span>
              )}
              {lastCacheSource === 'L2_HIT' && (
                <span className="text-cyan-400 font-bold flex items-center gap-1">
                  <Zap className="w-3 h-3" /> L2 Hit (&lt;5ms)
                </span>
              )}
              {lastCacheSource === 'DB_QUERY' && (
                <span className="text-amber-400 font-bold flex items-center gap-1">
                  <Database className="w-3 h-3" /> SQLx Query
                </span>
              )}
            </>
          )}
        </div>

        {/* Right Actions */}
        <div className="flex items-center gap-3">
          {/* Cart Trigger */}
          <button
            onClick={onOpenCart}
            className="relative p-2.5 rounded-xl bg-slate-800/80 hover:bg-slate-700/80 border border-slate-700/80 text-slate-200 transition-all active:scale-95"
          >
            <ShoppingBag className="w-5 h-5 text-indigo-400" />
            {cartCount > 0 && (
              <span className="absolute -top-1.5 -right-1.5 bg-gradient-to-r from-indigo-500 to-purple-600 text-white text-xs font-bold w-5 h-5 rounded-full flex items-center justify-center border-2 border-slate-900 animate-bounce">
                {cartCount}
              </span>
            )}
          </button>

          {/* User Profile / Auth Button */}
          {user ? (
            <div className="flex items-center gap-2 bg-slate-900/90 border border-slate-800 rounded-xl p-1.5 pl-3">
              <button
                onClick={() => setActiveView('auth')}
                className="flex items-center gap-2 text-sm text-left"
              >
                {user.role === 'StoreAdmin' ? (
                  <ShieldCheck className="w-4 h-4 text-purple-400" />
                ) : (
                  <UserCheck className="w-4 h-4 text-indigo-400" />
                )}
                <div>
                  <p className="font-bold text-xs text-slate-200 leading-tight">{user.full_name}</p>
                  <p className="text-[10px] text-indigo-400 font-mono leading-tight">{user.role}</p>
                </div>
              </button>
              <button
                onClick={onLogout}
                title="Logout"
                className="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-rose-400 transition-colors ml-1"
              >
                <LogOut className="w-4 h-4" />
              </button>
            </div>
          ) : (
            <Button
              onClick={() => setActiveView('auth')}
              className="text-xs"
            >
              <UserCheck className="w-4 h-4" /> Sign In / Register
            </Button>
          )}
        </div>
      </div>
    </header>
  );
};
