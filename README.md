# CardClaws

Interactive digital business card platform (RedClaw Systems LLC). Monorepo with
three surfaces:

- `cardclaws-backend/` — Rust/Axum API (Cargo workspace).
- `cardclaws-mobile/` — React Native Expo app.
- `cardclaws-profile/` — Astro web profile.

## Mobile (Expo)

```bash
cd cardclaws-mobile
npm install
npm run typecheck   # tsc --noEmit
npm test            # pure-logic unit suites (no simulator needed)
EXPO_PUBLIC_API_BASE=http://localhost:8080 npm start   # run in a simulator
```

See the implementation plan in `~/.claude/plans/` and the PRD for full scope.

## Backend — local development

Prerequisites: Rust (stable), Docker.

```bash
# 1. Start Postgres + Redis (Postgres on host port 5544, Redis on 6399).
docker compose up -d

# 2. Configure secrets (env-based locally; Infisical in prod).
export DATABASE_URL="postgres://cardclaws:cardclaws@localhost:5544/cardclaws"
export REDIS_URL="redis://127.0.0.1:6399"
export JWT_SECRET="dev-only-change-me"
export IP_HASH_SECRET="dev-only-change-me"

# 3. Run the API (migrations run automatically on startup).
cd cardclaws-backend
cargo run -p cardclaws-api
```

### Tests

```bash
cd cardclaws-backend

# Unit tests (no database needed).
cargo test --workspace

# Integration tests need a Postgres; set TEST_DATABASE_URL.
export TEST_DATABASE_URL="postgres://cardclaws:cardclaws@localhost:5544/cardclaws"
cargo test --workspace
```

Integration tests skip themselves cleanly when `TEST_DATABASE_URL` is unset.

### CI gates

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/loc-lint.sh      # no source file > 1300 lines (PRD §14)
bash scripts/policy-grep.sh   # no stub/placeholder markers (PRD §15.4)
```

## Status

- **B0** Monorepo + CI scaffolding — done.
- **B1** Backend foundation (config, db, auth, api) — done. `/v1/auth/*` fully
  tested (register, login + lockout, magic link, refresh rotation, logout,
  Apple sign-in verification).
- **B2** Card data layer + assets — done. `/v1/cards` CRUD (create, list, get,
  replace, patch, archive, publish w/ per-tier active-card limit, duplicate,
  public get-by-handle), R2 presigned uploads + delete, vCard export.
  (PNG export deferred to the card-render milestone.)
- **Web profile** (`cardclaws-profile`, Astro SSR) — done + verified end-to-end.
  `cardclaws.com/[handle]` renders the card hero (click-to-flip) + sticky
  contact-actions bar; Save Contact downloads a client-built RFC 6350 vCard and
  pings `contact_save`; SSR forwards the visitor IP so `profile_visit` is
  attributed correctly. `npm run build` + `astro check` clean.
- **B3** Mobile app (Expo) — built. Auth (login/register), card list + create,
  builder v1 (background/text/logo layers, palette, undo/redo), and the
  Skia/Reanimated card viewer (3D flip + entry + ambient drift). Logic core
  unit-tested (22 tests: handle validation, k-means palette, vCard, undo/redo
  store, QR); full app type-checks. Component/E2E runs need a simulator.
- **B4** Apple Wallet + analytics (backend) — done.

### Phase 4 (in progress)

- **Teams data layer (backend)** — done. `teams` + `team_memberships`
  (migration 0003) with owner/admin/member roles. `POST /v1/teams` (Team-tier
  gated), `GET /v1/teams/{id}`, `GET|POST /v1/teams/{id}/members`,
  `DELETE …/members/{userId}`. Add-by-email (existing accounts), seat-cap
  enforcement (402), role-based authorization (admin to mutate; non-members get
  404 so teams aren't enumerable); owner can't be removed.

### Phase 3

- **Tier-gate enforcement (backend)** — done. `Tier` capability model
  (pro-layers, geo-analytics, custom-domain, retention). Pro-only layer types
  (video/particle/animatedGradient) rejected on card create/replace/patch for
  Free; geo analytics gated to Pro+. Tier is read from the DB (authoritative),
  so a billing-webhook upgrade applies immediately. Returns 402 `tier_limit`.
- **Billing webhook → tier (backend)** — done. `POST /v1/webhooks/revenuecat`
  (shared-secret auth in the `Authorization` header, constant-time compare) maps
  RevenueCat lifecycle events to `users.tier` (purchase→pro/team/enterprise,
  cancellation/expiration→free). Unknown users are a 2xx no-op.

### Phase 2

- **Share system (backend)** — done. `POST /v1/cards/{id}/share` mints an opaque
  12-char base62 token; `GET /v1/s/{token}` records the modality-attributed
  event (qr→qr_scan, nfc→nfc_tap, …) and 302-redirects to the profile;
  `GET /v1/cards/{id}/share-links` lists them. Events carry the `share_token`
  correlation, and `GET /v1/cards/{id}/analytics/feed` returns the chronological
  feed.
- **Analytics rollups + geo (backend)** — done. Idempotent hourly rollup
  (`ON CONFLICT DO UPDATE`) into `analytics_rollups_hourly`, run by a Tokio
  background task; `GET /v1/cards/{id}/analytics/geo` (country breakdown). Geo
  lookup is behind a `GeoResolver` trait — real MaxMind under the `geoip`
  feature, no-op otherwise; the raw IP is resolved then discarded (only the
  hash + coarse country/city are stored).
- **Mobile share + analytics + layers** — done (type-checked, logic unit-tested).
  `ShapeLayer` type + layer reordering in `cardStore`; QR rendering (`QRCodeView`
  from the on-device matrix); `CardFace` renders shape + qr layers; share/
  analytics API clients; `ShareSheet` (QR overlay + OS share sheet); per-card
  analytics dashboard (summary + top countries + activity feed).
- **Google Wallet (backend + mobile link)** — done. `cardclaws-wallet::google`
  builds the inline GenericObject and an RS256-signed `savetowallet` JWT
  (`Rs256Signer` via `jsonwebtoken`, fake signer for tests); wired at
  `POST /v1/cards/{id}/wallet/google` → `{ saveUrl }`. Mobile share sheet has an
  "Add to Google Wallet" action that opens the save URL.
- **Profile depth + contact form** — done + verified live. Web profile renders
  About (bio) + Links sections + a contact form; `POST /v1/profile/{handle}/contact`
  validates, rate-limits, records a `contact_form_submission`, and emails the
  owner (Resend). End-to-end: mobile edits profile → web renders it → form
  submits to the owner.
- **Builder depth (mobile)** — done. `LayerPropertySheet` (per-type edit:
  text/color/fill/opacity + delete), `LayerOrderPanel` (reorder front↔back,
  select), `ProfileEditor` (bio + up to 12 links), shape-layer creation — all
  undoable via `cardStore` (profile/link actions unit-tested).
- **Remaining:** NFC tap + native Add-to-Apple-Wallet (need native modules + a
  dev build); Android build; portfolio/testimonials (Pro).

Backend tests: **67 passing** (unit + integration). Run with a live Postgres
(`TEST_DATABASE_URL`) to exercise the integration suite.

### Apple pass signing

The pass pipeline is structurally complete and tested with a fake signer. For
real, device-loadable passes build with the feature and supply the cert
material:

```bash
export APPLE_PASS_P12_BASE64="$(base64 -i PassType.p12)"
export APPLE_PASS_P12_PASSWORD="…"
export APPLE_WWDR_PEM="$(cat AppleWWDRCAG4.pem)"
cargo run -p cardclaws-api --features apple-signing
```
