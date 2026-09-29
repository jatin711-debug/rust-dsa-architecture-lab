-- Match the order list's filter and deterministic sort. This also prepares
-- for keyset pagination on (created_at, id).
CREATE INDEX IF NOT EXISTS idx_orders_user_created_id
ON orders (user_id, created_at DESC, id DESC);
