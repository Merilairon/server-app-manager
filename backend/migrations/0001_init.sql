-- Server App Manager initial schema.
-- Skeleton only: tables and seed rows are defined so the schema is ready
-- for P0 auth work. The backend does not yet read from these tables.

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username        TEXT NOT NULL UNIQUE,
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'user',
    status          TEXT NOT NULL DEFAULT 'active',
    last_login      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS roles (
    name          TEXT PRIMARY KEY,
    permissions   JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed roles matching roles/roles.yaml.
INSERT INTO roles (name, permissions)
VALUES
    ('user',  '["read:apps","install:apps"]'::jsonb),
    ('admin', '["admin:all"]'::jsonb)
ON CONFLICT (name) DO NOTHING;

-- Seed a default admin user with a placeholder bcrypt hash.
-- Replace this hash before any real deployment.
INSERT INTO users (username, email, password_hash, role)
VALUES
    ('admin', 'admin@example.com', '$2b$12$REPLACE_ME_WITH_A_REAL_BCRYPT_HASH', 'admin')
ON CONFLICT (username) DO NOTHING;
