# tsx — Production, Publish & Deployment Plan

> Written 2026-03-17. Covers everything needed to take the monorepo from a
> working development state to a live, observable, production deployment.

---

## 1. Services Overview

| Service | Stack | Host | URL |
|---------|-------|------|-----|
| `tsx-registry` (Rust API) | Axum + SQLx + Neon PostgreSQL | **Shuttle.dev** | `tsx-registry.shuttleapp.rs` |
| `registry-web` (dashboard) | TanStack Start + Nitro | **Vercel** | `registry.tsx.dev` |
| `docs` (documentation) | TanStack Start + MDX + Nitro | **Vercel** | `docs.tsx.dev` |
| PostgreSQL | Neon serverless | Neon cloud | (internal) |
| Tarball storage | Filesystem (Shuttle persistent) | Shuttle.dev | `/data/tarballs/` |

> Shuttle free tier: 3 projects, shared infrastructure. No credit card required.
> Upgrade to Shuttle Pro ($20/mo) for dedicated resources when traffic grows.

---

## 2. Environment Variables

### 2.1 `crates/registry-server` — Shuttle secrets

Shuttle reads from `Secrets.toml` (production) and `Secrets.dev.toml` (local dev, gitignored).

Set production secrets with the CLI (**never hardcode in committed files**):

```sh
# Run from crates/registry-server/
shuttle secret set DATABASE_URL "postgresql://neondb_owner:...@neon.tech/neondb?sslmode=require"
shuttle secret set TSX_REGISTRY_API_KEY "your-strong-secret-here"
shuttle secret set DATA_DIR "./data"
```

| Key | Required | Description |
|-----|----------|-------------|
| `DATABASE_URL` | yes | Neon PostgreSQL connection string |
| `TSX_REGISTRY_API_KEY` | no | Admin + publish bearer token (open if unset) |
| `DATA_DIR` | no | Tarball directory (default `./data`) |

### 2.2 `apps/registry-web` — Vercel environment

Set in Vercel project dashboard → Settings → Environment Variables:

```
VITE_REGISTRY_URL      https://tsx-registry.shuttleapp.rs
VITE_REGISTRY_API_KEY  <same value as TSX_REGISTRY_API_KEY>
DATABASE_URL           <same Neon URL — better-auth session tables>
BETTER_AUTH_SECRET     <random 32+ char secret>
BETTER_AUTH_URL        https://registry.tsx.dev
```

### 2.3 `apps/docs` — Vercel environment

```
VITE_REGISTRY_URL  https://tsx-registry.shuttleapp.rs
```

---

## 3. Database Setup (Neon)

The Neon database is already provisioned. Connection string:
```
postgresql://neondb_owner:...@ep-red-shadow-am07694x-pooler.c-5.us-east-1.aws.neon.tech/neondb?sslmode=require
```

- **Registry tables** (`packages`, `versions`, `download_logs`, `audit_log`) — created automatically by `tsx-registry` on first boot via `sqlx::migrate!`
- **Auth tables** (`user`, `session`, `account`) — managed by `better-auth` in `registry-web`
- Never run `drizzle-kit push` against registry tables; only `tsx-registry` owns them

---

## 4. Shuttle.dev — Registry Server

### 4.1 Install Shuttle CLI

```sh
# Windows (PowerShell)
iwr https://www.shuttle.dev/install-win | iex

# macOS / Linux
curl -sSfL https://www.shuttle.dev/install | bash

# Verify
shuttle --version
```

### 4.2 Login and create project

```sh
shuttle login                    # opens browser for GitHub OAuth
shuttle project create tsx-registry   # one-time project creation
```

### 4.3 Configure secrets (production)

```sh
cd crates/registry-server
shuttle secret set DATABASE_URL         "postgresql://neondb_owner:npg_...@neon.tech/neondb?sslmode=require"
shuttle secret set TSX_REGISTRY_API_KEY "your-strong-secret"
shuttle secret set DATA_DIR             "./data"
```

### 4.4 Local development

`Secrets.dev.toml` (already created, gitignored) contains the Neon dev DB:

```sh
cd crates/registry-server
shuttle run          # starts locally at http://localhost:8000
```

### 4.5 Deploy

```sh
cd crates/registry-server
shuttle deploy       # builds from source, deploys to Shuttle cloud
```

The deployed URL will be: `https://tsx-registry.shuttleapp.rs`

### 4.6 Health check

```sh
curl https://tsx-registry.shuttleapp.rs/health
# → {"status":"ok","version":"0.1.0"}
```

### 4.7 View logs

```sh
shuttle logs --follow --app tsx-registry
```

### 4.8 Scaling (Shuttle Pro)

When traffic outgrows the free tier:
1. Upgrade to Shuttle Pro: `shuttle upgrade`
2. Dedicated CPU + memory
3. Custom domain: `shuttle domain add api.tsx.dev --app tsx-registry`

---

## 5. Vercel — Registry Web

### 5.1 First deploy

```sh
cd apps/registry-web
npx vercel --prod
```

Or connect the GitHub repo in Vercel dashboard:

- **Root directory**: `apps/registry-web`
- **Framework preset**: Other
- **Build command**: `NITRO_PRESET=vercel bun run build`
- **Output directory**: `.vercel/output`
- **Install command**: `bun install`

The `apps/registry-web/vercel.json` handles this automatically when Vercel imports the project.

### 5.2 Custom domain

In Vercel project → Settings → Domains → Add `registry.tsx.dev`

### 5.3 Environment variables

Add from §2.2 in Vercel project → Settings → Environment Variables.
Mark `BETTER_AUTH_SECRET` and `DATABASE_URL` as **Sensitive**.

---

## 6. Vercel — Docs

### 6.1 First deploy

```sh
cd apps/docs
npx vercel --prod
```

Or in Vercel dashboard (second separate project):

- **Root directory**: `apps/docs`
- **Build command**: `NITRO_PRESET=vercel bun run build`
- **Output directory**: `.vercel/output`
- **Install command**: `bun install`

### 6.2 Custom domain

`docs.tsx.dev` → Vercel project → Settings → Domains.

---

## 7. Pre-Launch Checklist

### Security

- [ ] `TSX_REGISTRY_API_KEY` set in Shuttle secrets (never in committed files)
- [ ] `BETTER_AUTH_SECRET` set in Vercel (never committed)
- [ ] Admin endpoints return 401 without API key
- [ ] DOMPurify sanitizes all README HTML before render
- [ ] Rate limiter active (10 publishes/min per IP)
- [ ] CORS configured (`allow_origin(Any)` acceptable for public read API)

### Data integrity

- [ ] Neon DB connection tested: `psql $DATABASE_URL -c '\dt'`
- [ ] `tsx-registry` starts, runs migrations, logs "Database migrations applied"
- [ ] `DATA_DIR` writable: `shuttle logs` shows no directory errors
- [ ] Tarball slug format `@scope__name` tested (no collisions)

### Functional smoke tests

- [ ] `GET /health` → 200
- [ ] `GET /v1/stats` → 200 (zero counts on fresh DB)
- [ ] Publish a test package via CLI: `tsx publish --api-key $KEY my-test-pkg`
- [ ] Download: `GET /v1/packages/my-test-pkg/1.0.0/tarball`
- [ ] Search: `GET /v1/search?q=test`
- [ ] Admin audit log with correct key → 200
- [ ] Admin without key → 401
- [ ] Registry web dashboard loads and shows packages
- [ ] OAuth login (GitHub) completes in `registry-web`
- [ ] Docs site renders MDX pages

### Observability

- [ ] `shuttle logs` shows structured output
- [ ] Uptime monitor pointing at `https://tsx-registry.shuttleapp.rs/health`
- [ ] Vercel Analytics enabled for both apps

---

## 8. CI/CD — GitHub Actions

### Current jobs (`.github/workflows/ci.yml`)

| Job | Trigger | What it does |
|-----|---------|-------------|
| `rust` | push/PR | `cargo clippy --all`, `cargo fmt --check`, `cargo build --release -p tsx` |
| `web` | push/PR | `bun install` (workspace), `bun --cwd apps/registry-web build`, `tsc --noEmit` |
| `e2e` | push/PR | Playwright tests |
| `lighthouse` | push/PR | Lighthouse CI |
| `storybook` | push/PR | `bun run build-storybook` |

### Add deploy jobs (on merge to `main`)

```yaml
deploy-registry:
  needs: [rust]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: shuttle-hq/deploy-action@main
      with:
        working-directory: crates/registry-server
        shuttle-api-key: ${{ secrets.SHUTTLE_API_KEY }}

deploy-registry-web:
  needs: [web, e2e]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: amondnet/vercel-action@v25
      with:
        vercel-token:   ${{ secrets.VERCEL_TOKEN }}
        vercel-org-id:  ${{ secrets.VERCEL_ORG_ID }}
        vercel-project-id: ${{ secrets.VERCEL_REGISTRY_WEB_PROJECT_ID }}
        working-directory: apps/registry-web
        vercel-args: '--prod'

deploy-docs:
  needs: [web]
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: amondnet/vercel-action@v25
      with:
        vercel-token:   ${{ secrets.VERCEL_TOKEN }}
        vercel-org-id:  ${{ secrets.VERCEL_ORG_ID }}
        vercel-project-id: ${{ secrets.VERCEL_DOCS_PROJECT_ID }}
        working-directory: apps/docs
        vercel-args: '--prod'
```

### Required GitHub secrets

| Secret | How to get |
|--------|-----------|
| `SHUTTLE_API_KEY` | `shuttle auth key` |
| `VERCEL_TOKEN` | vercel.com → Account Settings → Tokens |
| `VERCEL_ORG_ID` | `vercel teams ls` or `.vercel/project.json` |
| `VERCEL_REGISTRY_WEB_PROJECT_ID` | `.vercel/project.json` in `apps/registry-web` |
| `VERCEL_DOCS_PROJECT_ID` | `.vercel/project.json` in `apps/docs` |

---

## 9. Domain & DNS

```
tsx-registry.shuttleapp.rs   (auto-assigned by Shuttle, no DNS needed)

# Custom domain (after go-live):
api.tsx.dev         CNAME  tsx-registry.shuttleapp.rs    (Shuttle custom domain)
registry.tsx.dev    CNAME  cname.vercel-dns.com           (Vercel)
docs.tsx.dev        CNAME  cname.vercel-dns.com           (Vercel)
```

---

## 10. Rollback Procedure

### Registry server (Shuttle)

```sh
# List deployments
shuttle deployment list --app tsx-registry

# Roll back to a specific deployment ID
shuttle deployment redeploy <deployment-id> --app tsx-registry
```

### Web apps (Vercel)

Vercel Dashboard → Project → Deployments → find previous → **Promote to Production**

### Database

No automated down-migration. Procedure:
1. Roll back binary immediately (above)
2. Write a corrective forward migration `000N_revert_<description>.sql`
3. Deploy with the next release

---

## 11. Step-by-Step: First Deployment

Run these commands in order:

```sh
# 1. Install tools (once)
iwr https://www.shuttle.dev/install-win | iex    # Windows
npm i -g vercel

# 2. Shuttle: login + create project
shuttle login
cd crates/registry-server
shuttle project create tsx-registry

# 3. Set production secrets
shuttle secret set DATABASE_URL         "postgresql://neondb_owner:npg_dpSjK8D9qBCl@ep-red-shadow-am07694x-pooler.c-5.us-east-1.aws.neon.tech/neondb?channel_binding=require&sslmode=require"
shuttle secret set TSX_REGISTRY_API_KEY "GENERATE_A_STRONG_SECRET_HERE"
shuttle secret set DATA_DIR             "./data"

# 4. Deploy registry server
shuttle deploy
# → https://tsx-registry.shuttleapp.rs

# 5. Smoke-test the API
curl https://tsx-registry.shuttleapp.rs/health

# 6. Deploy registry-web to Vercel
cd ../../apps/registry-web
vercel --prod
# Set VITE_REGISTRY_URL=https://tsx-registry.shuttleapp.rs in Vercel dashboard

# 7. Deploy docs to Vercel
cd ../docs
vercel --prod

# 8. Add GitHub secrets for CI auto-deploy (§8 above)
```

---

## 12. Monitoring & Observability

| Tool | What | Alert on |
|------|------|---------|
| `shuttle logs --follow` | Structured JSON logs | Error volume spikes |
| Uptime Robot (free) | `GET /health` every 5 min | Any non-200 |
| Neon console | Query latency, connections | Latency > 200ms avg |
| Vercel Analytics | Web Vitals | LCP > 2.5s, CLS > 0.1 |

### Log query pattern (Shuttle JSON logs)

```sh
shuttle logs --app tsx-registry 2>&1 | grep '"level":"WARN"'
```

---

## 13. CLI Binary Release

When the `tsx` CLI is ready:

```sh
# Bump version
sed -i 's/version = "0.1.0"/version = "0.2.0"/' crates/cli/Cargo.toml

# Tag
git tag v0.2.0 && git push origin v0.2.0
```

Add a GitHub Actions `release` job:

```yaml
release:
  if: startsWith(github.ref, 'refs/tags/v')
  runs-on: ubuntu-latest
  strategy:
    matrix:
      target: [x86_64-unknown-linux-musl, aarch64-apple-darwin, x86_64-pc-windows-msvc]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with: { targets: "${{ matrix.target }}" }
    - run: cargo build --release -p tsx --target ${{ matrix.target }}
    - uses: softprops/action-gh-release@v2
      with:
        files: target/${{ matrix.target }}/release/tsx*
```

---

## 14. Post-Launch Backlog

| Priority | Task | Notes |
|----------|------|-------|
| High | Rate limiter persistence (DB) | Replace in-memory HashMap; survives restarts |
| High | CLI binary release pipeline | GitHub Actions cross-compile + GitHub Releases |
| High | Tarball CDN (Cloudflare R2) | Move off Shuttle local disk; free 10 GB/mo |
| Medium | Prometheus `/metrics` endpoint | `axum-prometheus` crate → Grafana Cloud |
| Medium | `utoipa` OpenAPI annotations | Auto-generate `packages/api-types` types |
| Medium | `cargo test -p tsx-registry` in CI | Needs Neon branch DB per PR |
| Medium | Package ownership transfer API | `PATCH /v1/packages/:name/owner` |
| Low | Download stats webhook | Notify authors on milestone counts |
| Low | Package deprecation flag | Soft-delete with redirect to successor |
| Low | better-auth email magic-link | Email fallback for GitHub OAuth |
