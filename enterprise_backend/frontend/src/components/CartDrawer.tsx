import React, { useState } from 'react';
import { CartItem, OrderResponse } from '../types';
import { X, Trash2, Plus, Minus, CheckCircle, ArrowRight, ShoppingBag } from 'lucide-react';

interface CartDrawerProps {
  isOpen: boolean;
  cart: CartItem[];
  onClose: () => void;
  onUpdateQuantity: (productId: string, delta: number) => void;
  onRemoveItem: (productId: string) => void;
  onCheckout: () => Promise<OrderResponse>;
}

export const CartDrawer: React.FC<CartDrawerProps> = ({
  isOpen,
  cart,
  onClose,
  onUpdateQuantity,
  onRemoveItem,
  onCheckout,
}) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [placedOrder, setPlacedOrder] = useState<OrderResponse | null>(null);

  if (!isOpen) return null;

  const totalCents = cart.reduce((sum, item) => sum + item.product.price_cents * item.quantity, 0);

  const handleCheckoutSubmit = async () => {
    setError(null);
    setLoading(true);
    try {
      const order = await onCheckout();
      setPlacedOrder(order);
    } catch (err: any) {
      setError(err.message || 'Checkout failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 overflow-hidden bg-slate-950/80 backdrop-blur-md animate-fade-in flex justify-end">
      <div className="w-full max-w-md bg-slate-900 border-l border-slate-800 h-full flex flex-col justify-between shadow-2xl relative">

        {/* Header */}
        <div className="p-6 border-b border-slate-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <ShoppingBag className="w-5 h-5 text-indigo-400" />
            <h3 className="text-lg font-bold text-white">Your Shopping Cart</h3>
          </div>
          <button
            onClick={() => { setPlacedOrder(null); onClose(); }}
            className="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-4">
          {placedOrder ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 bg-emerald-500/20 text-emerald-400 rounded-full flex items-center justify-center mx-auto mb-4 border border-emerald-500/30">
                <CheckCircle className="w-8 h-8 animate-bounce" />
              </div>
              <h4 className="text-2xl font-bold text-white mb-1">Order Placed Successfully!</h4>
              <p className="text-xs text-slate-400 font-mono mb-4">Atomic SQLx Transaction Committed</p>

              <div className="p-4 rounded-xl bg-slate-800/80 border border-slate-700 text-left font-mono text-xs text-slate-300 space-y-2 mb-6">
                <p><span className="text-slate-500">Order ID:</span> {placedOrder.id}</p>
                <p><span className="text-slate-500">Status:</span> <span className="text-emerald-400 font-bold">{placedOrder.status}</span></p>
                <p><span className="text-slate-500">Total:</span> ${(placedOrder.total_amount_cents / 100).toFixed(2)}</p>
              </div>

              <button
                onClick={() => { setPlacedOrder(null); onClose(); }}
                className="w-full py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-sm transition-all"
              >
                Continue Shopping
              </button>
            </div>
          ) : cart.length === 0 ? (
            <div className="text-center py-16 text-slate-500">
              <ShoppingBag className="w-12 h-12 mx-auto mb-3 text-slate-700" />
              <p className="text-base font-semibold text-slate-400">Cart is Empty</p>
              <p className="text-xs text-slate-500 mt-1">Add hardware products from the catalog</p>
            </div>
          ) : (
            <>
              {error && (
                <div className="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs font-mono">
                  {error}
                </div>
              )}

              {cart.map((item) => (
                <div
                  key={item.product.id}
                  className="p-4 rounded-xl bg-slate-800/60 border border-slate-700/60 flex items-center justify-between"
                >
                  <div className="flex-1 pr-3">
                    <h5 className="font-bold text-sm text-white">{item.product.name}</h5>
                    <p className="text-xs font-mono text-indigo-400 mt-0.5">
                      ${(item.product.price_cents / 100).toFixed(2)} each
                    </p>
                  </div>

                  {/* Quantity Controls */}
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => onUpdateQuantity(item.product.id, -1)}
                      className="p-1 rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200"
                    >
                      <Minus className="w-3.5 h-3.5" />
                    </button>
                    <span className="font-mono text-sm text-white font-bold w-6 text-center">
                      {item.quantity}
                    </span>
                    <button
                      onClick={() => onUpdateQuantity(item.product.id, 1)}
                      className="p-1 rounded-lg bg-slate-700 hover:bg-slate-600 text-slate-200"
                    >
                      <Plus className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => onRemoveItem(item.product.id)}
                      className="p-1.5 rounded-lg text-slate-500 hover:text-rose-400 transition-colors ml-2"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </>
          )}
        </div>

        {/* Footer Checkout Button */}
        {!placedOrder && cart.length > 0 && (
          <div className="p-6 border-t border-slate-800 bg-slate-900/90">
            <div className="flex items-center justify-between mb-4 font-mono">
              <span className="text-xs text-slate-400 uppercase">Subtotal</span>
              <span className="text-2xl font-black text-white">
                ${(totalCents / 100).toFixed(2)}
              </span>
            </div>

            <button
              onClick={handleCheckoutSubmit}
              disabled={loading}
              className="w-full py-3.5 rounded-xl bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-400 hover:to-teal-500 text-white font-bold text-sm transition-all shadow-lg shadow-emerald-500/20 active:scale-95 flex items-center justify-center gap-2"
            >
              {loading ? (
                <span className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin" />
              ) : (
                <>
                  Checkout via Atomic SQLx Transaction <ArrowRight className="w-4 h-4" />
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
