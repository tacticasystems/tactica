CREATE TABLE users (
    id UUID PRIMARY KEY,

    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,

    display_name TEXT,
    icon_url TEXT,
    banner_url TEXT,
    biography TEXT,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_superuser BOOLEAN NOT NULL DEFAULT FALSE,

    password_hash TEXT,
    totp_secret TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE units (
    id UUID PRIMARY KEY,

    slug TEXT NOT NULL UNIQUE,

    display_name TEXT,
    icon_url TEXT,
    banner_url TEXT,
    biography TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE unit_ranks (
    id UUID PRIMARY KEY,

    unit_id UUID NOT NULL REFERENCES units (id) ON DELETE CASCADE,

    slug TEXT NOT NULL,

    display_name TEXT,
    icon_url TEXT,
    description TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (unit_id, slug)
);

CREATE TABLE unit_memberships (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units (id) ON DELETE CASCADE,
    rank_id UUID NOT NULL REFERENCES unit_ranks (id),

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (user_id, unit_id)
);

CREATE TABLE unit_roles (
    id UUID PRIMARY KEY,

    unit_id UUID NOT NULL REFERENCES units (id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    description TEXT,

    permissions BIGINT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (unit_id, display_name)
);

CREATE TABLE unit_settings (
    unit_id UUID PRIMARY KEY REFERENCES units (id) ON DELETE CASCADE,

    discord_guild_id TEXT,
    discord_guild_joined_at TIMESTAMP,

    initial_rank_id UUID NOT NULL REFERENCES unit_ranks (id),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by UUID REFERENCES users (id) ON DELETE SET NULL
);
