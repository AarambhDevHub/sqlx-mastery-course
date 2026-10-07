-- Migration 1 (Down): Drop users, orders, and employees schema

DROP TABLE IF EXISTS employees;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS user_role;
