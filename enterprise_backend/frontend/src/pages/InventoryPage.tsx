import React, { useState } from 'react';
import { Product, User } from '../types';
import { Button } from '../components/ui/button';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Badge } from '../components/ui/badge';
import { Layers, Plus, RefreshCw, AlertTriangle, CheckCircle, Edit, Package, ShieldAlert } from 'lucide-react';
import { api } from '../api/client';

interface InventoryPageProps {
  products: Product[];
  user: User | null;
  onRefresh: () => void;
}

export const InventoryPage: React.FC<InventoryPageProps> = ({ products, user, onRefresh }) => {
  const [editingProduct, setEditingProduct] = useState<Product | null>(null);
  const [editPrice, setEditPrice] = useState('');
  const [editStock, setEditStock] = useState('');
  const [saving, setSaving] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  if (!user || user.role !== 'StoreAdmin') {
    return (
      <div className="max-w-2xl mx-auto py-16 px-4 text-center">
        <Card className="border-rose-500/30">
          <CardHeader>
            <div className="w-16 h-16 bg-rose-500/10 text-rose-400 border border-rose-500/20 rounded-2xl flex items-center justify-center mx-auto mb-2">
              <ShieldAlert className="w-8 h-8" />
            </div>
            <CardTitle className="text-2xl font-bold text-rose-300">Access Restricted</CardTitle>
            <CardDescription className="text-slate-400">
              The Inventory Control Dashboard requires <strong>StoreAdmin</strong> role permissions.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-xs font-mono text-slate-400 mb-6">
              Please sign in using an account with <code className="text-purple-400">StoreAdmin</code> privileges (or use the one-click Admin demo button on the Sign In page).
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const handleEditClick = (p: Product) => {
    setEditingProduct(p);
    setEditPrice((p.price_cents / 100).toString());
    setEditStock(p.stock_quantity.toString());
    setMsg(null);
  };

  const handleSaveUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingProduct) return;
    setSaving(true);
    setMsg(null);

    try {
      const price_cents = Math.round(parseFloat(editPrice) * 100);
      const stock_quantity = parseInt(editStock, 10);

      await api.createProduct({
        name: editingProduct.name,
        description: editingProduct.description,
        price_cents,
        stock_quantity,
      });

      setMsg(`Updated stock for '${editingProduct.name}'. L1/L2 cache automatically invalidated.`);
      setEditingProduct(null);
      onRefresh();
    } catch (err: any) {
      setMsg(`Error: ${err.message}`);
    } finally {
      setSaving(false);
    }
  };

  const totalStock = products.reduce((acc, item) => acc + item.stock_quantity, 0);
  const lowStockItems = products.filter((item) => item.stock_quantity <= 3);

  return (
    <div className="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8 animate-fade-in space-y-6">

      {/* Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-3xl font-black text-white">Inventory Control Dashboard</h2>
            <Badge variant="purple">StoreAdmin View</Badge>
          </div>
          <p className="text-xs text-slate-400 font-mono mt-1">
            Real-time stock management with immediate L1 (Moka) & L2 (Redis) cache invalidation
          </p>
        </div>

        <Button variant="outline" onClick={onRefresh} className="font-mono text-xs">
          <RefreshCw className="w-4 h-4" /> Refresh Inventory
        </Button>
      </div>

      {/* Metrics Banner */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="p-4 border-indigo-500/20">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-xs text-slate-400 font-mono uppercase">Total Unique Products</span>
              <p className="text-3xl font-black text-white font-mono">{products.length}</p>
            </div>
            <Package className="w-8 h-8 text-indigo-400 opacity-60" />
          </div>
        </Card>

        <Card className="p-4 border-emerald-500/20">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-xs text-slate-400 font-mono uppercase">Total Physical Units</span>
              <p className="text-3xl font-black text-emerald-400 font-mono">{totalStock}</p>
            </div>
            <Layers className="w-8 h-8 text-emerald-400 opacity-60" />
          </div>
        </Card>

        <Card className="p-4 border-amber-500/20">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-xs text-slate-400 font-mono uppercase">Low Stock Warnings</span>
              <p className="text-3xl font-black text-amber-400 font-mono">{lowStockItems.length}</p>
            </div>
            <AlertTriangle className="w-8 h-8 text-amber-400 opacity-60" />
          </div>
        </Card>
      </div>

      {msg && (
        <div className="p-4 rounded-xl bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-mono text-xs flex items-center gap-2">
          <CheckCircle className="w-4 h-4 text-indigo-400 shrink-0" />
          <span>{msg}</span>
        </div>
      )}

      {/* Inventory Table */}
      <Card>
        <CardHeader>
          <CardTitle>Catalog Inventory Management</CardTitle>
          <CardDescription>Click edit on any product to update stock level and price in PostgreSQL</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-slate-800 text-xs font-mono text-slate-400 uppercase">
                  <th className="py-3 px-4">Product Details</th>
                  <th className="py-3 px-4">Price ($)</th>
                  <th className="py-3 px-4">Stock Level</th>
                  <th className="py-3 px-4">Status</th>
                  <th className="py-3 px-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60 text-sm">
                {products.map((p) => {
                  const isOutOfStock = p.stock_quantity <= 0;
                  const isLow = p.stock_quantity > 0 && p.stock_quantity <= 3;

                  return (
                    <tr key={p.id} className="hover:bg-slate-800/40 transition-colors">
                      <td className="py-3.5 px-4">
                        <p className="font-bold text-white">{p.name}</p>
                        <p className="text-xs text-slate-400 truncate max-w-xs">{p.description}</p>
                        <p className="text-[10px] font-mono text-slate-500 mt-0.5">ID: {p.id}</p>
                      </td>
                      <td className="py-3.5 px-4 font-mono font-bold text-gradient">
                        ${(p.price_cents / 100).toFixed(2)}
                      </td>
                      <td className="py-3.5 px-4 font-mono font-bold text-white">
                        {p.stock_quantity} units
                      </td>
                      <td className="py-3.5 px-4">
                        {isOutOfStock ? (
                          <Badge variant="rose">Out of Stock</Badge>
                        ) : isLow ? (
                          <Badge variant="amber">Low Stock</Badge>
                        ) : (
                          <Badge variant="emerald">Healthy</Badge>
                        )}
                      </td>
                      <td className="py-3.5 px-4 text-right">
                        <Button size="sm" variant="outline" onClick={() => handleEditClick(p)}>
                          <Edit className="w-3.5 h-3.5" /> Adjust Stock
                        </Button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Quick Edit Modal */}
      {editingProduct && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-md">
          <Card className="w-full max-w-md border-purple-500/30">
            <CardHeader>
              <CardTitle>Adjust Stock: {editingProduct.name}</CardTitle>
              <CardDescription>Updating stock invalidates both L1 Moka and L2 Redis catalog caches</CardDescription>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSaveUpdate} className="space-y-4">
                <div className="space-y-1.5">
                  <Label>Unit Price ($ USD)</Label>
                  <Input
                    type="number"
                    step="0.01"
                    required
                    value={editPrice}
                    onChange={(e) => setEditPrice(e.target.value)}
                  />
                </div>

                <div className="space-y-1.5">
                  <Label>New Stock Quantity</Label>
                  <Input
                    type="number"
                    required
                    value={editStock}
                    onChange={(e) => setEditStock(e.target.value)}
                  />
                </div>

                <div className="flex gap-2 pt-2">
                  <Button type="button" variant="outline" onClick={() => setEditingProduct(null)} className="w-1/2">
                    Cancel
                  </Button>
                  <Button type="submit" disabled={saving} variant="secondary" className="w-1/2">
                    {saving ? 'Saving...' : 'Save Stock Update'}
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
};
