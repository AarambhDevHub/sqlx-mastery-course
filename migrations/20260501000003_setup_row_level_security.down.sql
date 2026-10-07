-- Migration 3 (Down): Drop Row-Level Security documents table

DROP POLICY IF EXISTS tenant_isolation ON documents;
DROP TABLE IF EXISTS documents;
