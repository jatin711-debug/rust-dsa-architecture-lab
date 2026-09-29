import React, { useEffect, useState, useCallback } from 'react';
import { api } from './api/client';
import { User, Product, HealthCheckResponse, CartItem, CacheSourceHeader } from './types';
import { Navbar, PageView } from './components/Navbar';
import { CacheStatsPanel } from './components/CacheStatsPanel';
import { ProductCatalog } from './components/ProductCatalog';
import { AuthPage } from './pages/AuthPage';
import { InventoryPage } from './pages/InventoryPage';
import { OrdersPage } from './pages/OrdersPage';
import { DiagnosticsPage } from './pages/DiagnosticsPage';
import { CartDrawer } from './components/CartDrawer';
import { Sparkles, Terminal, Rocket } from 'lucide-react';
import { Button } from './components/ui/button';

export const App: React.FC = () => {
  const [activeView, setActiveView] = useState<PageView>('catalog');
  const [user, setUser] = useState<User | null>(null);
  const [health, setHealth] = useState<HealthCheckResponse | null>(null);
  const [products, setProducts] = useState<Product[]>([]);
  const [lastCacheSource, setLastCacheSource] = useState<CacheSourceHeader | null>(null);
  const [cart, setCart] = useState<CartItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [isCartOpen, setIsCartOpen] = useState(false);

  // Poll Health & Cache Metrics
  const fetchHealth = useCallback(async () => {
    try {
      const data = await api.getHealth();
      setHealth(data);
    } catch {
      setHealth(null);
    }
  }, []);

  // Fetch Products & extract X-Cache-Status header
  const fetchProducts = useCallback(async () => {
    setLoading(true);
    try {
      const { data, cacheSource } = await api.getProducts();
      setProducts(data);
      setLastCacheSource(cacheSource);
    } catch (err) {
      console.error('Failed to fetch products', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Fetch Current User Profile if JWT exists
  useEffect(() => {
    if (localStorage.getItem('jwt_token')) {
      api.getMe().then(setUser).catch(() => localStorage.removeItem('jwt_token'));
    }
  }, []);

  useEffect(() => {
    fetchHealth();
    fetchProducts();
    const interval = setInterval(fetchHealth, 5000);
    return () => clearInterval(interval);
  }, [fetchHealth, fetchProducts]);

  // Seed default enterprise products if catalog is empty
  const seedSampleProducts = async () => {
    if (!user || user.role !== 'StoreAdmin') {
      setActiveView('auth');
      return;
    }
    try {
      await api.createProduct({
        name: 'Quantum Neocortex AI Server Unit',
        description: 'Ultra-low latency inference server equipped with dual Liquid-cooled GPUs.',
        price_cents: 299900,
        stock_quantity: 12,
      });
      await api.createProduct({
        name: 'Nexus Fiber Mesh Router X9000',
        description: 'Multi-gigabit encrypted hardware routing node for low-latency microservices.',
        price_cents: 49900,
        stock_quantity: 25,
      });
      await api.createProduct({
        name: 'Titan NVMe RAID Storage Enclosure',
        description: 'PCIe 5.0 high density 100TB enterprise array for database acceleration.',
        price_cents: 149900,
        stock_quantity: 8,
      });
      fetchProducts();
      fetchHealth();
    } catch (err) {
      console.error('Failed to seed sample products', err);
    }
  };

  // Cart operations
  const handleAddToCart = (product: Product) => {
    setCart((prev) => {
      const existing = prev.find((item) => item.product.id === product.id);
      if (existing) {
        return prev.map((item) =>
          item.product.id === product.id ? { ...item, quantity: item.quantity + 1 } : item
        );
      }
      return [...prev, { product, quantity: 1 }];
    });
  };

  const handleUpdateCartQuantity = (productId: string, delta: number) => {
    setCart((prev) =>
      prev
        .map((item) => {
          if (item.product.id === productId) {
            const newQty = item.quantity + delta;
            return newQty > 0 ? { ...item, quantity: newQty } : null;
          }
          return item;
        })
        .filter(Boolean) as CartItem[]
    );
  };

  const handleRemoveCartItem = (productId: string) => {
    setCart((prev) => prev.filter((item) => item.product.id !== productId));
  };

  const handleCheckout = async () => {
    if (!user) {
      setActiveView('auth');
      throw new Error('Please sign in to place an order');
    }
    const orderItems = cart.map((item) => ({
      product_id: item.product.id,
      quantity: item.quantity,
    }));

    const orderRes = await api.createOrder(orderItems);
    setCart([]);
    fetchProducts();
    fetchHealth();
    return orderRes;
  };

  const handleLogout = () => {
    localStorage.removeItem('jwt_token');
    setUser(null);
  };

  const totalCartItems = cart.reduce((sum, item) => sum + item.quantity, 0);

  return (
    <div className="min-h-screen bg-[#0b0f19] text-slate-100 flex flex-col">
      <Navbar
        activeView={activeView}
        setActiveView={setActiveView}
        user={user}
        health={health}
        cartCount={totalCartItems}
        lastCacheSource={lastCacheSource}
        onOpenCart={() => setIsCartOpen(true)}
        onLogout={handleLogout}
      />

      <main className="flex-1">
        {activeView === 'auth' && (
          <AuthPage
            user={user}
            onSuccess={(u) => {
              setUser(u);
              setActiveView('catalog');
              fetchProducts();
            }}
            onLogout={handleLogout}
          />
        )}

        {activeView === 'inventory' && (
          <InventoryPage
            products={products}
            user={user}
            onRefresh={() => {
              fetchProducts();
              fetchHealth();
            }}
          />
        )}

        {activeView === 'orders' && (
          <OrdersPage
            user={user}
            onOpenAuth={() => setActiveView('auth')}
          />
        )}

        {activeView === 'diagnostics' && (
          <DiagnosticsPage
            health={health}
            onRefresh={fetchHealth}
          />
        )}

        {activeView === 'catalog' && (
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 w-full">
            {/* Hero Section */}
            <div className="mb-8 relative glass-card rounded-3xl p-8 sm:p-10 border border-slate-800 overflow-hidden">
              <div className="absolute top-0 right-0 w-96 h-96 bg-gradient-to-br from-indigo-500/10 via-purple-500/10 to-transparent rounded-full blur-3xl pointer-events-none" />

              <div className="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
                <div>
                  <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 text-xs font-mono mb-4">
                    <Sparkles className="w-3.5 h-3.5" /> Axum 0.8 • PostgreSQL SQLx • Moka L1 & Redis L2
                  </div>
                  <h1 className="text-3xl sm:text-4xl lg:text-5xl font-black text-white tracking-tight">
                    Enterprise <span className="text-gradient">Hardware & API</span> Catalog
                  </h1>
                  <p className="text-slate-400 text-sm sm:text-base mt-2 max-w-2xl leading-relaxed">
                    Click <strong>"Refetch & Test Cache"</strong> to watch response speed up from <strong className="text-amber-400">PostgreSQL</strong> to <strong className="text-cyan-400">L2 Redis (&lt;5ms)</strong> and <strong className="text-emerald-400">L1 Moka (&lt;1ms)</strong>!
                  </p>
                </div>

                <div className="flex flex-col sm:flex-row gap-3 w-full md:w-auto">
                  <Button onClick={fetchProducts} className="font-semibold text-xs">
                    <Rocket className="w-4 h-4" /> Refetch & Test Cache
                  </Button>

                  {products.length === 0 && (
                    <Button variant="secondary" onClick={seedSampleProducts} className="font-semibold text-xs">
                      <Terminal className="w-4 h-4" /> Seed Sample Products
                    </Button>
                  )}
                </div>
              </div>
            </div>

            {/* Cache Stats Visualizer Bar */}
            <CacheStatsPanel health={health} onRefresh={fetchHealth} />

            {/* Catalog Grid */}
            <ProductCatalog
              products={products}
              user={user}
              loading={loading}
              onAddToCart={handleAddToCart}
              onCreateProduct={async (dto) => {
                await api.createProduct(dto);
                fetchProducts();
                fetchHealth();
              }}
            />
          </div>
        )}
      </main>

      {/* Slide-over Cart Drawer */}
      <CartDrawer
        isOpen={isCartOpen}
        cart={cart}
        onClose={() => setIsCartOpen(false)}
        onUpdateQuantity={handleUpdateCartQuantity}
        onRemoveItem={handleRemoveCartItem}
        onCheckout={handleCheckout}
      />

      <footer className="glass-panel border-t border-slate-800/80 py-6 mt-12 text-center text-xs text-slate-500 font-mono">
        <p>Enterprise Backend Architecture • Built with Axum, SQLx, Redis, Moka, React 19 & TypeScript 7</p>
      </footer>
    </div>
  );
};
