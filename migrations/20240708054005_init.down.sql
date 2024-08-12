-- Add down migration script here

-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS sessions
