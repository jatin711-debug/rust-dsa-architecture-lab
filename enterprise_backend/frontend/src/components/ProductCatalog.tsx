import React, { useState } from 'react';
import { Product, User } from '../types';
import { ShoppingCart, Plus, Search, Layers, Tag, CheckCircle, AlertTriangle, Package, X } from 'lucide-react';

interface ProductCatalogProps {
  products: Product[];
  user: User | null;
  loading: boolean;
  onAddToCart: (product: Product) => void;
  onCreateProduct: (dto: { name: string; description?: string; price_cents: number; stock_quantity: number }) => Promise<void>;
}

export const ProductCatalog: React.FC<ProductCatalogProps> = ({
  products,
  user,
  loading,
  onAddToCart,
  onCreateProduct,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [priceDollars, setPriceDollars] = useState('');
  const [stockQuantity, setStockQuantity] = useState('10');
  const [creating, setCreating] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);

  const filteredProducts = products.filter((p) =>
    p.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    p.description.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleCreateSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setCreateError(null);
    setCreating(true);

    try {
      const priceCents = Math.round(parseFloat(priceDollars) * 100);
      const stock = parseInt(stockQuantity, 10);

      if (isNaN(priceCents) || priceCents < 0) {
        throw new Error('Invalid price value');
      }
      if (isNaN(stock) || stock < 0) {
        throw new Error('Invalid stock quantity');
      }

      await onCreateProduct({
        name,
        description,
        price_cents: priceCents,
        stock_quantity: stock,
      });

      setName('');
      setDescription('');
      setPriceDollars('');
      setStockQuantity('10');
      setIsModalOpen(false);
    } catch (err: any) {
      setCreateError(err.message || 'Failed to create product');
    } finally {
      setCreating(false);
    }
  };

  return (
    <div>
      {/* Search & Actions Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-6">

        {/* Search Input */}
        <div className="relative w-full sm:w-80">
          <Search className="absolute left-3.5 top-3 w-4 h-4 text-slate-500" />
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder="Search enterprise hardware & software..."
            className="w-full pl-10 pr-4 py-2.5 bg-slate-900/80 border border-slate-800 rounded-xl text-slate-200 placeholder-slate-500 text-sm focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all"
          />
        </div>

        {/* Admin Add Product Modal Trigger */}
        {user?.role === 'StoreAdmin' && (
          <button
            onClick={() => setIsModalOpen(true)}
            className="w-full sm:w-auto px-4 py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white font-medium text-sm transition-all shadow-lg shadow-purple-600/30 flex items-center justify-center gap-2 active:scale-95"
          >
            <Plus className="w-4 h-4" /> Add Product (Admin)
          </button>
        )}
      </div>

      {/* Product Grid */}
      {loading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {[1, 2, 3].map((n) => (
            <div key={n} className="glass-card rounded-2xl p-6 h-64 animate-pulse" />
          ))}
        </div>
      ) : filteredProducts.length === 0 ? (
        <div className="glass-panel rounded-2xl p-12 text-center my-8">
          <Package className="w-12 h-12 text-slate-600 mx-auto mb-3" />
          <h4 className="text-lg font-bold text-slate-300">No Products Found</h4>
          <p className="text-sm text-slate-500 mt-1">Try adjusting your search or add a product as StoreAdmin</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredProducts.map((product) => {
            const isOutOfStock = product.stock_quantity <= 0;
            const isLowStock = product.stock_quantity > 0 && product.stock_quantity <= 3;

            return (
              <div
                key={product.id}
                className="glass-card glass-card-hover rounded-2xl p-6 flex flex-col justify-between relative group"
              >
                <div>
                  {/* Stock Status Badge */}
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-[10px] font-mono uppercase px-2.5 py-1 rounded-md bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 flex items-center gap-1">
                      <Tag className="w-3 h-3" /> Hardware
                    </span>
                    {isOutOfStock ? (
                      <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-rose-500/20 text-rose-400 border border-rose-500/30 flex items-center gap-1">
                        <AlertTriangle className="w-3 h-3" /> Out of Stock
                      </span>
                    ) : isLowStock ? (
                      <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-amber-500/20 text-amber-300 border border-amber-500/30 flex items-center gap-1">
                        <AlertTriangle className="w-3 h-3" /> Only {product.stock_quantity} left
                      </span>
                    ) : (
                      <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
                        <CheckCircle className="w-3 h-3" /> In Stock ({product.stock_quantity})
                      </span>
                    )}
                  </div>

                  <h3 className="text-xl font-bold text-white group-hover:text-indigo-300 transition-colors">
                    {product.name}
                  </h3>
                  <p className="text-xs text-slate-400 mt-2 line-clamp-2 leading-relaxed">
                    {product.description || 'No product description provided.'}
                  </p>
                </div>

                <div className="mt-6 pt-4 border-t border-slate-800 flex items-center justify-between">
                  <div>
                    <span className="text-xs text-slate-500 uppercase font-mono block">Price</span>
                    <span className="text-2xl font-black text-gradient">
                      ${(product.price_cents / 100).toFixed(2)}
                    </span>
                  </div>

                  <button
                    onClick={() => onAddToCart(product)}
                    disabled={isOutOfStock}
                    className="px-4 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 disabled:hover:bg-indigo-600 text-white font-semibold text-xs transition-all shadow-lg shadow-indigo-600/30 active:scale-95 flex items-center gap-1.5"
                  >
                    <ShoppingCart className="w-4 h-4" /> Add to Cart
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* StoreAdmin Create Product Modal */}
      {isModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-md">
          <div className="relative w-full max-w-lg glass-card rounded-2xl p-6 sm:p-8 border border-purple-500/30 shadow-2xl">
            <button
              onClick={() => setIsModalOpen(false)}
              className="absolute top-4 right-4 p-2 rounded-xl text-slate-400 hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>

            <h3 className="text-xl font-bold text-white mb-1 flex items-center gap-2">
              <Layers className="w-5 h-5 text-purple-400" /> Create Product Catalog Item
            </h3>
            <p className="text-xs text-slate-400 mb-6 font-mono">
              Triggers instant database insertion & L1/L2 cache invalidation
            </p>

            {createError && (
              <div className="mb-4 p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs font-mono">
                {createError}
              </div>
            )}

            <form onSubmit={handleCreateSubmit} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Product Name</label>
                <input
                  type="text"
                  required
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. Quantum AI Accelerator Card"
                  className="w-full px-4 py-2.5 bg-slate-900 border border-slate-800 rounded-xl text-slate-100 text-sm focus:border-purple-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Description</label>
                <textarea
                  rows={3}
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Enterprise hardware specification..."
                  className="w-full px-4 py-2.5 bg-slate-900 border border-slate-800 rounded-xl text-slate-100 text-sm focus:border-purple-500 focus:outline-none"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Price ($ USD)</label>
                  <input
                    type="number"
                    step="0.01"
                    required
                    value={priceDollars}
                    onChange={(e) => setPriceDollars(e.target.value)}
                    placeholder="499.99"
                    className="w-full px-4 py-2.5 bg-slate-900 border border-slate-800 rounded-xl text-slate-100 text-sm focus:border-purple-500 focus:outline-none font-mono"
                  />
                </div>

                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Stock Quantity</label>
                  <input
                    type="number"
                    required
                    value={stockQuantity}
                    onChange={(e) => setStockQuantity(e.target.value)}
                    placeholder="25"
                    className="w-full px-4 py-2.5 bg-slate-900 border border-slate-800 rounded-xl text-slate-100 text-sm focus:border-purple-500 focus:outline-none font-mono"
                  />
                </div>
              </div>

              <button
                type="submit"
                disabled={creating}
                className="w-full py-3 rounded-xl bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white font-semibold text-sm transition-all shadow-lg shadow-purple-600/30 mt-2"
              >
                {creating ? 'Creating Product...' : 'Publish Product to Catalog'}
              </button>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
