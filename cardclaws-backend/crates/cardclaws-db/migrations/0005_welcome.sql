-- AI Welcome (PRD §21 Phase 5, "scan experiences"): an optional generated
-- greeting shown when the public profile is scanned. Stored as JSONB:
--   { "kind": "image", "message": "...", "imageDataUrl": "data:image/png;base64,..." }
-- (image inlined as a data URL for the MVP; swaps to an R2 URL later).
ALTER TABLE cards ADD COLUMN welcome JSONB;
