-- Add up migration script here
CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name TEXT NOT NULL,
    email VARCHAR(254) NOT NULL,
    UNIQUE (name, email),
    password TEXT NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE
);

SELECT manage_updated_at('users');
