export type Role = 'Customer' | 'StoreAdmin';

export interface User {
  id: string;
  email: string;
  full_name: string;
  role: Role;
  created_at: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Product {
  id: string;
  name: string;
  description: string;
  price_cents: number;
  stock_quantity: number;
  created_at: string;
  updated_at: string;
}

export interface OrderItemInput {
  product_id: string;
  quantity: number;
}

export interface OrderItemResponse {
  id: string;
  product_id: string;
  unit_price_cents: number;
  quantity: number;
}

export interface OrderResponse {
  id: string;
  user_id: string;
  status: string;
  total_amount_cents: number;
  items: OrderItemResponse[];
  created_at: string;
}

export interface CartItem {
  product: Product;
  quantity: number;
}

export interface HealthCheckResponse {
  status: string;
  database: string;
  cache: {
    l1_hits: number;
    l2_hits: number;
    misses: number;
  };
  timestamp: string;
}

export type CacheSourceHeader = 'L1_HIT' | 'L2_HIT' | 'DB_QUERY' | 'UNKNOWN';
