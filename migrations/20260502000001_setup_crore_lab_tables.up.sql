-- Migration 4 (Up): Part 2 - 100 Crore Lab Tables, Extensions, Partitions, and Queues

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Lab Users table for billion-row stress benchmarks
CREATE TABLE lab_users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Counter table for fast O(1) row counts
CREATE TABLE counter_aggregates (
    id INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    user_count BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO counter_aggregates (id, user_count) VALUES (1, 0)
ON CONFLICT (id) DO NOTHING;

-- Trigger to maintain user_count in counter_aggregates
CREATE OR REPLACE FUNCTION trg_lab_users_count_func()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE counter_aggregates 
        SET user_count = user_count + 1, updated_at = now() 
        WHERE id = 1;
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE counter_aggregates 
        SET user_count = user_count - 1, updated_at = now() 
        WHERE id = 1;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_lab_users_count
AFTER INSERT OR DELETE ON lab_users
FOR EACH ROW EXECUTE FUNCTION trg_lab_users_count_func();

-- Range partitioned events table (time-series demonstration)
CREATE TABLE events (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE TABLE events_2026_01 PARTITION OF events
    FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

CREATE TABLE events_2026_02 PARTITION OF events
    FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

CREATE TABLE events_2026_03 PARTITION OF events
    FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

CREATE TABLE events_2026_04 PARTITION OF events
    FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

CREATE TABLE events_default PARTITION OF events DEFAULT;

-- Posts and Sharded Counters for Hot Row Contention Challenge
CREATE TABLE posts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title TEXT NOT NULL,
    view_count BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE counter_shards (
    post_id BIGINT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    shard_id INT NOT NULL,
    count BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (post_id, shard_id)
);

-- Transactional Job Queue table for SKIP LOCKED Challenge
CREATE TABLE jobs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    queue TEXT NOT NULL DEFAULT 'default',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jobs_status_created_at ON jobs (status, created_at);
