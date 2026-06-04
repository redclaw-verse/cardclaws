-- 0002_analytics.sql — raw event log and hourly rollups (PRD §9.3 / §18).
-- Added in B2 so the analytics write path (B4) has a home before week 7.

CREATE TABLE analytics_events (
    id          BIGSERIAL PRIMARY KEY,
    card_id     UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL,
    share_token TEXT REFERENCES share_links(token),
    -- SHA-256(ip + per-day salt). Never raw IP (§18.3).
    ip_hash     TEXT,
    country     TEXT,
    city        TEXT,
    user_agent  TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_analytics_card_time ON analytics_events(card_id, occurred_at DESC);

CREATE TABLE analytics_rollups_hourly (
    card_id     UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    hour        TIMESTAMPTZ NOT NULL,
    visits      INTEGER NOT NULL DEFAULT 0,
    qr_scans    INTEGER NOT NULL DEFAULT 0,
    nfc_taps    INTEGER NOT NULL DEFAULT 0,
    saves       INTEGER NOT NULL DEFAULT 0,
    link_clicks INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (card_id, hour)
);
