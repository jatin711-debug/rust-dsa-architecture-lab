import React, { useState } from 'react';
import { api } from '../api/client';
import { User } from '../types';
import { Button } from '../components/ui/button';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Badge } from '../components/ui/badge';
import { Lock, Mail, User as UserIcon, Shield, Eye, EyeOff, CheckCircle2, AlertCircle, Key } from 'lucide-react';

interface AuthPageProps {
  user: User | null;
  onSuccess: (user: User) => void;
  onLogout: () => void;
}

export const AuthPage: React.FC<AuthPageProps> = ({ user, onSuccess, onLogout }) => {
  const [tab, setTab] = useState<'login' | 'signup'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [fullName, setFullName] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (tab === 'login') {
        const res = await api.login({ email, password });
        localStorage.setItem('jwt_token', res.token);
        onSuccess(res.user);
      } else {
        const res = await api.register({ email, password, full_name: fullName });
        localStorage.setItem('jwt_token', res.token);
        onSuccess(res.user);
      }
    } catch (err: any) {
      setError(err.message || 'Authentication failed');
    } finally {
      setLoading(false);
    }
  };

  // If user is already signed in, show Profile Page
  if (user) {
    return (
      <div className="max-w-3xl mx-auto py-12 px-4 animate-fade-in">
        <Card className="border-indigo-500/30">
          <CardHeader className="text-center pb-2">
            <div className="w-16 h-16 bg-gradient-to-tr from-indigo-600 to-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-3 shadow-lg shadow-indigo-600/30">
              <Shield className="w-8 h-8 text-white" />
            </div>
            <CardTitle className="text-2xl font-bold">Authenticated User Profile</CardTitle>
            <CardDescription className="font-mono">Argon2 Password Hashing & Verified JWT Bearer Session</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6 pt-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800 space-y-1">
                <span className="text-xs text-slate-500 font-mono uppercase">Full Name</span>
                <p className="font-bold text-lg text-white">{user.full_name}</p>
              </div>
              <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800 space-y-1">
                <span className="text-xs text-slate-500 font-mono uppercase">Email Address</span>
                <p className="font-bold text-lg text-indigo-400 font-mono">{user.email}</p>
              </div>
              <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800 space-y-1">
                <span className="text-xs text-slate-500 font-mono uppercase">Role Permissions</span>
                <div>
                  <Badge variant={user.role === 'StoreAdmin' ? 'purple' : 'default'}>
                    {user.role}
                  </Badge>
                </div>
              </div>
              <div className="p-4 rounded-xl bg-slate-950/60 border border-slate-800 space-y-1">
                <span className="text-xs text-slate-500 font-mono uppercase">User ID (UUID)</span>
                <p className="font-mono text-xs text-slate-300 truncate">{user.id}</p>
              </div>
            </div>

            <div className="p-4 rounded-xl bg-indigo-950/30 border border-indigo-500/20 text-xs font-mono text-indigo-300 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Key className="w-4 h-4 text-indigo-400" />
                <span>JWT Token active in localStorage</span>
              </div>
              <Badge variant="emerald">Valid Session</Badge>
            </div>

            <Button variant="destructive" onClick={onLogout} className="w-full">
              Sign Out & Invalidate Session
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="max-w-md mx-auto py-12 px-4 animate-fade-in">
      <Card>
        <CardHeader className="text-center pb-2">
          <div className="w-12 h-12 bg-indigo-500/10 border border-indigo-500/20 rounded-2xl flex items-center justify-center mx-auto mb-2 text-indigo-400">
            <Lock className="w-6 h-6" />
          </div>
          <CardTitle className="text-2xl font-bold">
            {tab === 'login' ? 'Sign In to Account' : 'Register New Account'}
          </CardTitle>
          <CardDescription>
            {tab === 'login' ? 'Access your orders & administrative settings' : 'Create an enterprise customer or store admin profile'}
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Tab Switcher */}
          <div className="grid grid-cols-2 gap-1 p-1 bg-slate-950/80 rounded-xl border border-slate-800">
            <button
              type="button"
              onClick={() => { setTab('login'); setError(null); }}
              className={`py-2 text-xs font-bold rounded-lg transition-all ${
                tab === 'login' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-white'
              }`}
            >
              Sign In
            </button>
            <button
              type="button"
              onClick={() => { setTab('signup'); setError(null); }}
              className={`py-2 text-xs font-bold rounded-lg transition-all ${
                tab === 'signup' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-white'
              }`}
            >
              Register
            </button>
          </div>

          {/* Error Alert */}
          {error && (
            <div className="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs font-mono flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            {tab === 'signup' && (
              <div className="space-y-1.5">
                <Label htmlFor="full_name">Full Name</Label>
                <div className="relative">
                  <UserIcon className="absolute left-3.5 top-3.5 w-4 h-4 text-slate-500" />
                  <Input
                    id="full_name"
                    required
                    value={fullName}
                    onChange={(e) => setFullName(e.target.value)}
                    placeholder="Jane Doe"
                    className="pl-10"
                  />
                </div>
              </div>
            )}

            <div className="space-y-1.5">
              <Label htmlFor="email">Email Address</Label>
              <div className="relative">
                <Mail className="absolute left-3.5 top-3.5 w-4 h-4 text-slate-500" />
                <Input
                  id="email"
                  type="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="developer@enterprise.com"
                  className="pl-10"
                />
              </div>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="password">Password</Label>
              <div className="relative">
                <Lock className="absolute left-3.5 top-3.5 w-4 h-4 text-slate-500" />
                <Input
                  id="password"
                  type={showPassword ? 'text' : 'password'}
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••••••"
                  className="pl-10 pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3.5 top-3.5 text-slate-500 hover:text-slate-300"
                >
                  {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            <Button type="submit" disabled={loading} className="w-full">
              {loading ? 'Processing...' : (
                <>
                  <CheckCircle2 className="w-4 h-4" /> {tab === 'login' ? 'Authenticate' : 'Complete Registration'}
                </>
              )}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};
