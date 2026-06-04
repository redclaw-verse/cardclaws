-- 0004_team_templates.sql — central template library for teams (PRD §6.1.4,
-- Phase 4: admin-managed templates shared with team members).

CREATE TABLE team_templates (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id    UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    definition JSONB NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_team_templates_team ON team_templates(team_id);
