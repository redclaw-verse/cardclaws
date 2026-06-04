-- 0001_initial.sql — core tables (PRD §9.3) plus the cardclaws_sessions table
-- that §9.3 omitted but §6.8.1/§20.1 require.

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    handle        TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    tier          TEXT NOT NULL DEFAULT 'free'
                      CHECK (tier IN ('free', 'pro', 'team', 'enterprise')),
    -- Argon2id PHC string. NULL for accounts created purely via OAuth/magic link.
    password_hash TEXT,
    avatar_r2_key TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE cards (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    handle      TEXT NOT NULL UNIQUE,
    status      TEXT NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'active', 'archived')),
    definition  JSONB NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cards_owner ON cards(owner_id);
CREATE INDEX idx_cards_handle ON cards(handle) WHERE status = 'active';

-- Refresh-token sessions. The refresh token itself is never stored; only a
-- SHA-256 hash so a DB leak cannot be replayed. Rotated on every use (§20.1).
CREATE TABLE cardclaws_sessions (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    device_fingerprint TEXT,
    last_active_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at         TIMESTAMPTZ NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user ON cardclaws_sessions(user_id);

CREATE TABLE share_links (
    token       TEXT PRIMARY KEY,
    card_id     UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    modality    TEXT NOT NULL,
    campaign    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ
);

CREATE INDEX idx_share_links_card ON share_links(card_id);

CREATE TABLE wallet_registrations (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id           UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    platform          TEXT NOT NULL CHECK (platform IN ('apple', 'google')),
    device_library_id TEXT,
    push_token        TEXT,
    registered_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_wallet_reg_card ON wallet_registrations(card_id);
