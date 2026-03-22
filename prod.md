# TSX — Production Publishing & Release Plan

Three independently deployable artifacts: the `tsx` CLI binary, the registry server, and first-party packages.

---

## Artifacts

| Artifact | Crate / Dir | Ships to |
| --- | --- | --- |
| `tsx` binary | `crates/cli` | GitHub Releases, Homebrew, npm shim |
| Registry server | `crates/registry-server` | Render.com (tsx-tsnv.onrender.com) |
| First-party packages | `packages/*` | Self-hosted tsx registry |

---

## 1 — Registry Server

Already live. Redeploy by pushing to `main` — Render picks up `render.yaml` automatically.

### Required env vars (Render dashboard)

| Variable | Notes |
| --- | --- |
| `DATABASE_URL` | Neon PostgreSQL connection string |
| `TSX_REGISTRY_API_KEY` | `openssl rand -hex 32` — used for publish/admin auth |
| `DATA_DIR` | `/data` on Render persistent disk |
| `PORT` | Auto-set by Render |

### Pending DB work

The discovery query reads `versions.manifest` as JSONB. Apply once if not already present:

```sql
ALTER TABLE versions ADD COLUMN IF NOT EXISTS manifest JSONB DEFAULT '{}'::jsonb;
CREATE INDEX IF NOT EXISTS idx_versions_manifest ON versions USING GIN (manifest);
```

Run existing migrations if starting from a fresh DB:

```bash
psql "$DATABASE_URL" -f migrations/0001_initial_schema.sql
psql "$DATABASE_URL" -f migrations/0002_fts_and_rate_limits.sql
psql "$DATABASE_URL" -f migrations/0003_webhooks.sql
psql "$DATABASE_URL" -f migrations/0004_stars.sql
psql "$DATABASE_URL" -f migrations/0005_deprecation.sql
```

### Verify

```bash
curl https://tsx-tsnv.onrender.com/health
curl "https://tsx-tsnv.onrender.com/v1/discovery?npm=@tanstack/start,drizzle-orm"
curl "https://tsx-tsnv.onrender.com/v1/commands"
```

---

## 2 — First-Party Packages

Publish all six packages in `packages/` to the registry so `tsx stack init` can auto-discover them.

```bash
export TSX_REGISTRY_URL=https://tsx-tsnv.onrender.com
export TSX_TOKEN=<TSX_REGISTRY_API_KEY value>

for pkg in tanstack-start drizzle-pg drizzle-mysql drizzle-sqlite better-auth shadcn; do
  tsx package validate packages/$pkg
  tsx package publish packages/$pkg --registry $TSX_REGISTRY_URL --token $TSX_TOKEN
done
```

Verify discovery works after publish:

```bash
curl "$TSX_REGISTRY_URL/v1/discovery?npm=@tanstack/start,drizzle-orm,better-auth,shadcn-ui"
# Expected: all four npm names map to tsx packages
```

### Versioning

Bump `version` in `packages/<id>/manifest.json` before each publish. Use semantic versioning — breaking template changes = minor bump, fixes = patch.

---

## 3 — CLI Binary

### 3.1 Version bump

```bash
# 1. Edit version in crates/cli/Cargo.toml
# 2. Update CHANGELOG.md
git commit -m "chore: bump to vX.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

### 3.2 GitHub Actions release workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release
on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu  }
          - { os: macos-latest,   target: x86_64-apple-darwin       }
          - { os: macos-latest,   target: aarch64-apple-darwin      }
          - { os: windows-latest, target: x86_64-pc-windows-msvc    }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - run: cargo build --release --target ${{ matrix.target }} -p tsx
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/tsx*
```

### 3.3 Install methods

```bash
# Unix one-liner (after install script is written)
curl -fsSL https://tsx.dev/install.sh | sh

# Homebrew
brew install ateeq1999/tap/tsx

# npm shim
npm i -g tsx-cli
```

---

## 4 — Release Checklist

### Pre-release

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy -p tsx -- -D warnings` clean
- [ ] Version bumped in `Cargo.toml` and `CHANGELOG.md`
- [ ] Each first-party package passes `tsx package validate packages/<id>`
- [ ] Registry server healthy: `/health` → 200
- [ ] Discovery returns expected results for `@tanstack/start`, `drizzle-orm`, `better-auth`

### Release

- [ ] Tag pushed: `git tag vX.Y.Z && git push origin vX.Y.Z`
- [ ] GitHub Actions builds binaries for all four targets
- [ ] GitHub Release created with binaries attached
- [ ] First-party packages published to registry
- [ ] `tsx stack init` smoke-tested in a real TanStack Start project

### Post-release

- [ ] `tsx tui` shows installed packages in browser
- [ ] `tsx run add:schema --json '{"name":"products"}'` executes successfully
- [ ] Announce release

---

## 5 — Secrets

| Secret | Stored in | Used by |
| --- | --- | --- |
| `DATABASE_URL` | Render env | Registry server |
| `TSX_REGISTRY_API_KEY` | Render env | Registry server publish/admin auth |
| `GITHUB_TOKEN` | GitHub Actions (auto) | Release binary uploads |
| `TSX_TOKEN` | Local `.env` only | `tsx package publish` |

`Secrets.dev.toml` and `.env` are gitignored — never commit secrets.

---

## 6 — Rollback

### Registry server

Roll back via Render dashboard (one click), or:

```bash
git revert HEAD && git push origin main
```

### CLI binary

All GitHub Release versions are permanent. Users can pin by downloading a specific tag URL.

### Package version

Yank a bad version (users with it cached keep it; new installs skip it):

```bash
curl -X DELETE "$TSX_REGISTRY_URL/v1/packages/<id>/versions/<ver>" \
  -H "Authorization: Bearer $TSX_TOKEN"
```
