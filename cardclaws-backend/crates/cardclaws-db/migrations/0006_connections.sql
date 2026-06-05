-- "We Met" scan experience (PRD §21 Phase 5): when someone scans a card and
-- shares back, we capture the meeting (with coarse geo + time) so the owner gets
-- a "People I met" list. Raw IP is never stored — only the resolved country/city.
CREATE TABLE connections (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id     UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    owner_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    email       TEXT,
    note        TEXT,
    country     TEXT,
    city        TEXT,
    met_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_connections_card ON connections(card_id, met_at DESC);
CREATE INDEX idx_connections_owner ON connections(owner_id, met_at DESC);
