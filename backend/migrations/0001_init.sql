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

-- Default admin is created at first startup from ADMIN_USERNAME / ADMIN_PASSWORD.
-- Do not seed a real password hash here; the backend hashes at runtime.
