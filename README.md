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
- **B4** Apple Wallet + analytics (backend) — done. `cardclaws-wallet` crate
  (pass.json builder, SHA-1 manifest, strip render, zip packager, PKCS#7
  OpenSSL signer behind the `apple-signing` feature) wired at
  `POST /v1/cards/{id}/wallet/apple`. Analytics ingest + summary
  (`/v1/analytics/event`, `/v1/cards/{id}/analytics`) with daily-salted IP
  hashing; `profile_visit` recorded on public handle lookup.
  Remaining for B4: the Astro web profile (client surface) + mobile viewer/flip.

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
