-- Migration 2 (Up): Accounts, Triggers, Views, Materialized Views, and NOTIFY

CREATE TABLE accounts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    balance NUMERIC(12, 2) NOT NULL DEFAULT 0.00 CHECK (balance >= 0),
    version INT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Trigger to automatically update updated_at
CREATE OR REPLACE FUNCTION update_timestamp_func()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_timestamp_func();

CREATE TRIGGER trg_accounts_updated_at
BEFORE UPDATE ON accounts
FOR EACH ROW
EXECUTE FUNCTION update_timestamp_func();

-- Standard View
CREATE OR REPLACE VIEW user_order_details AS
SELECT 
    u.id AS user_id,
    u.username,
    u.email,
    COUNT(o.id) AS order_count,
    COALESCE(SUM(o.amount), 0.00) AS total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.username, u.email;

-- Materialized View with Unique Index for Concurrent Refresh
CREATE MATERIALIZED VIEW user_order_summaries AS
SELECT 
    u.id AS user_id,
    u.username,
    COUNT(o.id) AS order_count,
    COALESCE(SUM(o.amount), 0.00) AS total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.username;

CREATE UNIQUE INDEX idx_mv_user_order_summaries_user_id 
ON user_order_summaries (user_id);

-- LISTEN / NOTIFY Trigger on Orders
CREATE OR REPLACE FUNCTION notify_order_event_func()
RETURNS TRIGGER AS $$
DECLARE
    payload TEXT;
BEGIN
    payload := json_build_object(
        'order_id', NEW.id,
        'user_id', NEW.user_id,
        'amount', NEW.amount,
        'status', NEW.status
    )::text;
    PERFORM pg_notify('order_events', payload);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_order_notify
AFTER INSERT ON orders
FOR EACH ROW
EXECUTE FUNCTION notify_order_event_func();
