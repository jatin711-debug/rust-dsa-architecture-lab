import { AuthResponse, HealthCheckResponse, OrderResponse, Product, User, CacheSourceHeader } from '../types';

const API_BASE = '';

function getAuthHeader(): Record<string, string> {
  const token = localStorage.getItem('jwt_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export interface ApiResponse<T> {
  data: T;
  cacheSource: CacheSourceHeader;
}

export const api = {
  // Health & Diagnostics
  async getHealth(): Promise<HealthCheckResponse> {
    const res = await fetch(`${API_BASE}/health`);
    if (!res.ok) throw new Error('Backend offline');
    return res.json();
  },

  // Auth Endpoints
  async register(data: { email: string; password: string; full_name: string }): Promise<AuthResponse> {
    const res = await fetch(`${API_BASE}/api/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    const json = await res.json();
    if (!res.ok) throw new Error(json.message || 'Registration failed');
    return json;
  },

  async login(data: { email: string; password: string }): Promise<AuthResponse> {
    const res = await fetch(`${API_BASE}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    const json = await res.json();
    if (!res.ok) throw new Error(json.message || 'Login failed');
    return json;
  },

  async getMe(): Promise<User> {
    const res = await fetch(`${API_BASE}/api/v1/auth/me`, {
      headers: getAuthHeader(),
    });
    const json = await res.json();
    if (!res.ok) throw new Error(json.message || 'Unauthorized');
    return json;
  },

  // Product Catalog Endpoints (with X-Cache-Status extraction)
  async getProducts(): Promise<ApiResponse<Product[]>> {
    const res = await fetch(`${API_BASE}/api/v1/products`, {
      headers: getAuthHeader(),
    });
    const cacheHeader = (res.headers.get('X-Cache-Status') as CacheSourceHeader) || 'UNKNOWN';
    const data = await res.json();
    if (!res.ok) throw new Error('Failed to fetch products');
    return { data, cacheSource: cacheHeader };
  },

  async createProduct(dto: { name: string; description?: string; price_cents: number; stock_quantity: number }): Promise<Product> {
    const res = await fetch(`${API_BASE}/api/v1/products`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...getAuthHeader(),
      },
      body: JSON.stringify(dto),
    });
    const json = await res.json();
    if (!res.ok) throw new Error(json.message || 'Failed to create product');
    return json;
  },

  // Order Endpoints
  async createOrder(items: { product_id: string; quantity: number }[]): Promise<OrderResponse> {
    const res = await fetch(`${API_BASE}/api/v1/orders`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...getAuthHeader(),
      },
      body: JSON.stringify({ items }),
    });
    const json = await res.json();
    if (!res.ok) throw new Error(json.message || 'Checkout failed');
    return json;
  },

  async getUserOrders(): Promise<OrderResponse[]> {
    const res = await fetch(`${API_BASE}/api/v1/orders`, {
      headers: getAuthHeader(),
    });
    const json = await res.json();
    if (!res.ok) throw new Error('Failed to fetch orders');
    return json;
  },
};
