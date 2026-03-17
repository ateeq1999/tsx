# tsx — Production, Publish & Deployment Plan

> Written 2026-03-17. Covers everything needed to take the monorepo from a
> working development state to a live, observable, production deployment.

---

## 1. Services Overview

| Service | Stack | Host | URL target |
|---------|-------|------|-----------|
| `tsx-registry` (Rust API) | Axum + SQLx + PostgreSQL | Fly.io | `api.tsx.dev` |
| `registry-web` (dashboard) | TanStack Start + Nitro | Vercel | `registry.tsx.dev` |
| `docs` (documentation) | TanStack Start + MDX | Vercel | `docs.tsx.dev` |
| PostgreSQL | Neon (serverless) | Neon cloud | (internal) |
| Tarball storage | Fly.io persistent volume | Fly.io | `/data/tarballs/` |

---

## 2. Environment Variables

### 2.1 `crates/registry-server` (Fly.io secrets)

```
DATABASE_URL            postgresql://user:pass@neon-host/tsx_registry?sslmode=require
TSX_REGISTRY_API_KEY    <random 32+ char secret>  — admin endpoints + CLI publish
PORT                    8080  (set in fly.toml)
DATA_DIR                /data  (set in fly.toml)
LOG_FORMAT              json  — enables structured JSON output to Fly log drain
```

Set with:

```sh
fly secrets set DATABASE_URL="..." TSX_REGISTRY_API_KEY="..." LOG_FORMAT=json --app tsx-registry
```

### 2.2 `apps/registry-web` (Vercel environment)

```
VITE_REGISTRY_URL       https://api.tsx.dev
VITE_REGISTRY_API_KEY   <same value as TSX_REGISTRY_API_KEY>
DATABASE_URL            <same Neon URL — for better-auth session tables only>
BETTER_AUTH_SECRET      <random 32+ char secret>
BETTER_AUTH_URL         https://registry.tsx.dev
```

### 2.3 `apps/docs` (Vercel environment)

```
VITE_REGISTRY_URL       https://api.tsx.dev
```

---

## 3. Database Setup (Neon)

1. Create a Neon project: `tsx-registry` in region `us-east-1`.
2. Copy the connection string (pooled endpoint) to all services that need `DATABASE_URL`.
3. **Migrations are applied automatically** by `tsx-registry` at startup via `sqlx::migrate!`. No manual step needed for the registry tables.
4. **better-auth tables** (sessions, users, accounts) are managed by the `registry-web` app via Drizzle at startup. These live in the same database but in separate tables.

```
migrations/
├── 0001_initial_schema.sql   ← applied by tsx-registry on first boot
```

> Never run `drizzle-kit push` against the production database for the
> registry tables. Only `tsx-registry` binary owns those tables.

---

## 4. Fly.io — Registry Server

### 4.1 First deploy

```sh
# From repo root
fly launch --no-deploy --name tsx-registry --region iad
fly volumes create tsx_registry_data --size 10 --region iad --app tsx-registry
fly secrets set DATABASE_URL="..." TSX_REGISTRY_API_KEY="..." LOG_FORMAT=json --app tsx-registry
fly deploy --app tsx-registry
```

### 4.2 Subsequent deploys

```sh
fly deploy --app tsx-registry
```

The GitHub Actions `deploy-registry` job (see §8) handles this automatically on merge to `main`.

### 4.3 `fly.toml` highlights

```toml
app = "tsx-registry"
primary_region = "iad"

[build]
  dockerfile = "crates/registry-server/Dockerfile"

[env]
  PORT     = "8080"
  DATA_DIR = "/data"

[http_service]
  internal_port = 8080
  force_https   = true
  auto_stop_machines  = "stop"
  auto_start_machines = true
  min_machines_running = 0

  [http_service.concurrency]
    type       = "connections"
    hard_limit = 100
    soft_limit = 80

[[mounts]]
  source      = "tsx_registry_data"
  destination = "/data"

[[vm]]
  size   = "shared-cpu-1x"
  memory = "256mb"
```

### 4.4 Health check

```sh
curl https://api.tsx.dev/health
# → {"status":"ok","version":"0.1.0"}
```

### 4.5 Scaling

When traffic grows beyond the shared-cpu-1x tier:

1. `fly scale vm shared-cpu-2x --app tsx-registry`
2. `fly scale count 2 --app tsx-registry` for HA
3. `min_machines_running = 1` to eliminate cold starts

---

## 5. Vercel — Registry Web

### 5.1 First deploy

```sh
cd apps/registry-web
npx vercel --prod
```

Or connect the GitHub repo in Vercel dashboard:

- **Framework preset**: Other (Vite + Nitro)
- **Build command**: `bun --cwd apps/registry-web build`
- **Output directory**: `apps/registry-web/dist`
- **Install command**: `bun install` (run at repo root)

### 5.2 Environment variables (Vercel dashboard)

Add all variables from §2.2. Mark `BETTER_AUTH_SECRET` and `DATABASE_URL` as **Sensitive**.

### 5.3 Custom domain

`registry.tsx.dev` → add as a custom domain in the Vercel project.

---

## 6. Vercel — Docs

### 6.1 First deploy

```sh
cd apps/docs
npx vercel --prod
```

Or in Vercel dashboard:

- **Build command**: `bun --cwd apps/docs build`
- **Output directory**: `apps/docs/dist`
- **Install command**: `bun install`

### 6.2 Custom domain

`docs.tsx.dev` → Vercel project settings → Domains.

---

## 7. Pre-Launch Checklist

### Security

- [ ] `TSX_REGISTRY_API_KEY` set in Fly.io secrets (never hardcoded)
- [ ] `BETTER_AUTH_SECRET` set in Vercel (never committed)
- [ ] Admin endpoints return 401 without API key (`require_admin_key` fails closed)
- [ ] DOMPurify sanitizes all README HTML before render
- [ ] Rate limiter active (`RATE_LIMIT_MAX_REQUESTS=10/min` per IP)
- [ ] `force_https = true` in fly.toml
- [ ] CORS locked to known origins in Axum middleware

### Data integrity

- [ ] Neon DB created, connection string tested (`psql $DATABASE_URL -c '\dt'`)
- [ ] `tsx-registry` starts and runs migrations on first boot (check logs)
- [ ] Fly volume attached and `DATA_DIR=/data` accessible (`fly ssh console -a tsx-registry -C "ls /data"`)
- [ ] Tarball slug format uses `@scope__name` (no collisions)

### Functional smoke tests

- [ ] `GET /health` → 200
- [ ] `GET /v1/stats` → 200 with zero counts (fresh DB)
- [ ] Publish a test package via CLI: `tsx publish --api-key $KEY my-test-pkg`
- [ ] Download the test package: `GET /v1/packages/my-test-pkg/1.0.0/tarball`
- [ ] Search: `GET /v1/search?q=test`
- [ ] Admin audit log: `GET /v1/admin/audit-log` with correct key → 200
- [ ] Admin without key → 401
- [ ] Registry web dashboard loads
- [ ] OAuth login (GitHub) completes in `registry-web`

### Observability

- [ ] Fly log drain configured (Papertrail / Axiom / Datadog)
- [ ] `LOG_FORMAT=json` set — structured logs flowing
- [ ] Uptime monitor pointing at `https://api.tsx.dev/health`

---

## 8. CI/CD — GitHub Actions

### Current jobs (`.github/workflows/ci.yml`)

| Job | Trigger | What it does |
|-----|---------|-------------|
| `rust` | push/PR | `cargo clippy --all`, `cargo fmt --check`, `cargo build --release -p tsx` |
| `web` | push/PR | `bun install` (workspace), `bun --cwd apps/registry-web build`, `tsc --noEmit` |
| `e2e` | push/PR | Playwright tests against `registry-web` |
| `lighthouse` | push/PR | Lighthouse CI score gates |
| `storybook` | push/PR | `bun run build-storybook` |

### Add deploy jobs (recommended)

Extend `ci.yml` with deploy steps gated on `main` branch and CI passing:

```yaml
deploy-registry:
  needs: [rust]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: superfly/flyctl-actions/setup-flyctl@master
    - run: fly deploy --app tsx-registry --remote-only
      env:
        FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}

deploy-registry-web:
  needs: [web, e2e]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: npx vercel --prod --token ${{ secrets.VERCEL_TOKEN }}
      env:
        VERCEL_ORG_ID:     ${{ secrets.VERCEL_ORG_ID }}
        VERCEL_PROJECT_ID: ${{ secrets.VERCEL_REGISTRY_WEB_PROJECT_ID }}

deploy-docs:
  needs: [web]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: npx vercel --prod --token ${{ secrets.VERCEL_TOKEN }}
      env:
        VERCEL_ORG_ID:     ${{ secrets.VERCEL_ORG_ID }}
        VERCEL_PROJECT_ID: ${{ secrets.VERCEL_DOCS_PROJECT_ID }}
```

### Required GitHub secrets

```
FLY_API_TOKEN                 fly tokens create deploy -a tsx-registry
VERCEL_TOKEN                  vercel tokens create
VERCEL_ORG_ID                 vercel teams ls
VERCEL_REGISTRY_WEB_PROJECT_ID  vercel project ls (registry-web project)
VERCEL_DOCS_PROJECT_ID          vercel project ls (docs project)
```

---

## 9. Domain & DNS

```
api.tsx.dev         CNAME  tsx-registry.fly.dev        (Fly.io)
registry.tsx.dev    CNAME  cname.vercel-dns.com         (Vercel)
docs.tsx.dev        CNAME  cname.vercel-dns.com         (Vercel)
```

All three domains enforced HTTPS via Fly (`force_https = true`) and Vercel (automatic).

---

## 10. Rollback Procedure

### Registry server rollback

```sh
# List recent releases
fly releases --app tsx-registry

# Roll back to previous image
fly deploy --image registry.fly.io/tsx-registry:<previous-version> --app tsx-registry
```

### Web app rollback

In Vercel dashboard → Deployments → click any previous deployment → **Promote to Production**.

### Database rollback

There is no automated down-migration. If a schema change is breaking:

1. Roll back the binary immediately (§above).
2. Write a corrective forward migration (`000N_revert_<description>.sql`).
3. Deploy the corrective migration with the next release.

> Never delete or modify committed migration files. Only forward migrations.

---

## 11. Monitoring & Alerting

### Recommended setup

| Tool | What to monitor | Alert on |
|------|----------------|---------|
| Fly.io metrics | CPU, memory, HTTP error rate | >5% 5xx for 5 min |
| Uptime Robot (free) | `GET https://api.tsx.dev/health` | Any non-200 |
| Neon console | DB connections, query latency | >100ms avg query |
| Vercel Analytics | Web Vitals, error rate | LCP >2.5s, CLS >0.1 |

### Structured log fields (JSON mode)

```json
{"level":"INFO","package":"my-pkg","version":"1.0.0","author":"alice","ip":"1.2.3.4","message":"Package published"}
{"level":"WARN","ip":"1.2.3.4","count":11,"message":"Publish rate limit exceeded"}
```

Query in Fly log drain (Axiom example):

```
level="WARN" message="Publish rate limit exceeded"
```

---

## 12. Publish — CLI Release Process

When the `tsx` CLI binary is ready for a new release:

```sh
# Bump version in crates/cli/Cargo.toml
# Tag the release
git tag v0.2.0
git push origin v0.2.0
```

Add a `release` GitHub Actions job that:

1. Runs `cargo build --release -p tsx --target x86_64-unknown-linux-musl`
2. Cross-compiles for `aarch64-apple-darwin` and `x86_64-pc-windows-msvc`
3. Uploads binaries to the GitHub release via `gh release create`
4. Updates the `tsx.dev/install` script to point at the latest tag

### install.sh skeleton

```sh
#!/bin/sh
VERSION="$(curl -sf https://api.github.com/repos/ateeq1999/tsx/releases/latest | grep tag_name | cut -d'"' -f4)"
ARCH="$(uname -m)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
curl -fsSL "https://github.com/ateeq1999/tsx/releases/download/${VERSION}/tsx-${OS}-${ARCH}" \
  -o /usr/local/bin/tsx && chmod +x /usr/local/bin/tsx
```

---

## 13. Post-Launch Backlog

| Priority | Task | Notes |
|----------|------|-------|
| High | Rate limiter persistence in DB | Replace in-memory HashMap; prevents bypass via restart/IP rotation |
| High | Prometheus `/metrics` endpoint | Add `axum-prometheus` crate; connect to Grafana Cloud |
| High | CLI binary release pipeline | GitHub Actions cross-compile + GitHub Releases |
| Medium | `utoipa` OpenAPI annotations | Auto-generate `packages/api-types/src/index.ts` |
| Medium | `cargo test -p tsx-registry` CI | Needs test DB in CI (Neon branch per PR, or Docker pg) |
| Medium | Package ownership transfer API | `PATCH /v1/packages/:name/owner` endpoint |
| Medium | Tarball CDN (Cloudflare R2) | Move off Fly volume; enable CDN caching for downloads |
| Low | Download stats webhook | Notify authors on milestone download counts |
| Low | Package deprecation flag | Soft-delete with redirect to successor |
| Low | `better-auth` email magic-link | Replace GitHub-only OAuth with email fallback |
