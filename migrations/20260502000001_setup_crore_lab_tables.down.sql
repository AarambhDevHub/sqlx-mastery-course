-- Migration 4 (Down): Drop Part 2 lab tables and partitions

DROP TABLE IF EXISTS jobs;
DROP TABLE IF EXISTS counter_shards;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS events;
DROP TRIGGER IF EXISTS trg_lab_users_count ON lab_users;
DROP FUNCTION IF EXISTS trg_lab_users_count_func;
DROP TABLE IF EXISTS counter_aggregates;
DROP TABLE IF EXISTS lab_users;
