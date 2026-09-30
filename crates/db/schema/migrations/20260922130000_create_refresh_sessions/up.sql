CREATE TABLE refresh_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX refresh_sessions_user_id_idx ON refresh_sessions (user_id);
CREATE INDEX refresh_sessions_expires_at_idx ON refresh_sessions (expires_at);

-- Retain consumed hashes until the session expires to detect replay.
CREATE TABLE refresh_tokens (
    token_hash TEXT PRIMARY KEY CHECK (token_hash ~ '^[0-9a-f]{64}$'),
    session_id UUID NOT NULL REFERENCES refresh_sessions (id) ON DELETE CASCADE,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX refresh_tokens_session_id_idx ON refresh_tokens (session_id);
CREATE UNIQUE INDEX refresh_tokens_one_active_per_session
    ON refresh_tokens (session_id) WHERE consumed_at IS NULL;
