-- Migration 2 (Down): Drop database logic objects

DROP TRIGGER IF EXISTS trg_order_notify ON orders;
DROP FUNCTION IF EXISTS notify_order_event_func;

DROP MATERIALIZED VIEW IF EXISTS user_order_summaries;
DROP VIEW IF EXISTS user_order_details;

DROP TRIGGER IF EXISTS trg_accounts_updated_at ON accounts;
DROP TRIGGER IF EXISTS trg_users_updated_at ON users;
DROP FUNCTION IF EXISTS update_timestamp_func;

DROP TABLE IF EXISTS accounts;
