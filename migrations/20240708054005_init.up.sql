-- Add up migration script here

-- Your SQL goes here
CREATE TABLE users (
    id         SERIAL       PRIMARY KEY,
    email      TEXT         NOT NULL UNIQUE,
    username   TEXT         NOT NULL UNIQUE,
    password   TEXT         NOT NULL,
    is_verify  BOOLEAN      NOT NULL DEFAULT false,
    img        VARCHAR(255),
    created_at TIMESTAMPTZ  NOT NULL,
    updated_at TIMESTAMPTZ
);

CREATE TABLE sessions (
    session_token BYTEA             PRIMARY KEY,
    id integer REFERENCES users(id) ON DELETE CASCADE
);