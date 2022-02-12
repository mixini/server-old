-- Add up migration script here
CREATE TYPE role AS ENUM ('admin', 'moderator', 'maintainer', 'creator', 'contributor', 'member');

CREATE TABLE users (
    id UUID PRIMARY KEY NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
    name TEXT NOT NULL,
    email VARCHAR(254) NOT NULL,
    UNIQUE (name, email),
    role role NOT NULL DEFAULT 'member',
    password TEXT NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE
);

SELECT manage_updated_at('users');
