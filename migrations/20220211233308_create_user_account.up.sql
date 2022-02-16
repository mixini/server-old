-- Add up migration script here
CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'maintainer', 'creator', 'contributor', 'member');

CREATE TABLE user_account (
    id UUID PRIMARY KEY NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,
    name TEXT NOT NULL,
    email VARCHAR(254) NOT NULL,
    UNIQUE (name, email),
    role user_role NOT NULL DEFAULT 'member',
    password TEXT NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE
);

SELECT manage_updated_at('user_account');
