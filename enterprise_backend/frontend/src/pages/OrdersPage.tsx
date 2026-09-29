import React, { useEffect, useState } from 'react';
import { OrderResponse, User } from '../types';
import { api } from '../api/client';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { ShoppingBag, RefreshCw, Clock, CheckCircle2, Package, ShieldCheck } from 'lucide-react';

interface OrdersPageProps {
  user: User | null;
  onOpenAuth: () => void;
}

export const OrdersPage: React.FC<OrdersPageProps> = ({ user, onOpenAuth }) => {
  const [orders, setOrders] = useState<OrderResponse[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchOrders = async () => {
    if (!user) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.getUserOrders();
      setOrders(data);
    } catch (err: any) {
      setError(err.message || 'Failed to load order history');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchOrders();
  }, [user]);

  if (!user) {
    return (
      <div className="max-w-2xl mx-auto py-16 px-4 text-center">
        <Card>
          <CardHeader>
            <div className="w-16 h-16 bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 rounded-2xl flex items-center justify-center mx-auto mb-2">
              <ShoppingBag className="w-8 h-8" />
            </div>
            <CardTitle className="text-2xl font-bold">Sign In to View Orders</CardTitle>
            <CardDescription>
              View orders placed via PostgreSQL atomic SQLx transactions.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={onOpenAuth} className="mt-2">
              Sign In to Account
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8 animate-fade-in space-y-6">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-3xl font-black text-white">Order Placement History</h2>
            <Badge variant="cyan">Atomic SQLx Transactions</Badge>
          </div>
          <p className="text-xs text-slate-400 font-mono mt-1">
            Orders processed with PostgreSQL row locking (`FOR UPDATE`) & ACID transaction rollback guarantees
          </p>
        </div>

        <Button variant="outline" onClick={fetchOrders} className="font-mono text-xs">
          <RefreshCw className="w-4 h-4" /> Refresh Orders
        </Button>
      </div>

      {loading ? (
        <div className="space-y-4">
          {[1, 2].map((n) => (
            <Card key={n} className="h-32 animate-pulse" />
          ))}
        </div>
      ) : error ? (
        <Card className="border-rose-500/30 p-6 text-rose-300 font-mono text-xs">
          {error}
        </Card>
      ) : orders.length === 0 ? (
        <Card className="text-center py-16">
          <Package className="w-12 h-12 text-slate-600 mx-auto mb-3" />
          <h4 className="text-lg font-bold text-slate-300">No Orders Found</h4>
          <p className="text-xs text-slate-500 mt-1">Add products to your cart and checkout to create your first order</p>
        </Card>
      ) : (
        <div className="space-y-4">
          {orders.map((order) => (
            <Card key={order.id} className="border-slate-800 hover:border-indigo-500/40 transition-all">
              <CardHeader className="pb-3 border-b border-slate-800/80">
                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-mono font-bold text-sm text-white">Order #{order.id.substring(0, 8)}...</span>
                      <Badge variant="emerald">{order.status}</Badge>
                    </div>
                    <p className="text-[10px] font-mono text-slate-500 mt-0.5">
                      Full ID: {order.id} • User ID: {order.user_id}
                    </p>
                  </div>

                  <div className="text-right font-mono">
                    <span className="text-xs text-slate-500 uppercase block">Total Amount</span>
                    <span className="text-xl font-extrabold text-gradient">
                      ${(order.total_amount_cents / 100).toFixed(2)}
                    </span>
                  </div>
                </div>
              </CardHeader>

              <CardContent className="pt-4">
                <p className="text-xs font-mono text-slate-400 mb-2 flex items-center gap-1">
                  <Clock className="w-3.5 h-3.5 text-indigo-400" /> Placed on: {new Date(order.created_at).toLocaleString()}
                </p>

                <div className="bg-slate-950/60 rounded-xl p-3 border border-slate-800/80 space-y-2">
                  <p className="text-xs font-mono text-slate-400 uppercase font-semibold">Ordered Line Items:</p>
                  {order.items.map((item) => (
                    <div key={item.id} className="flex items-center justify-between text-xs font-mono py-1 border-b border-slate-800/40 last:border-0">
                      <div className="flex items-center gap-2">
                        <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                        <span className="text-slate-200">Product ID: {item.product_id}</span>
                        <span className="text-slate-500">x{item.quantity}</span>
                      </div>
                      <span className="text-indigo-300 font-bold">${(item.unit_price_cents / 100).toFixed(2)} each</span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};
