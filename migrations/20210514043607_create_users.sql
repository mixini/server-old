-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name TEXT NOT NULL,
    email VARCHAR(254) NOT NULL,
    UNIQUE (name, email),
    password TEXT NOT NULL
);

SELECT manage_updated_at('users');
