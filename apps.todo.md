# apps.todo.md — Feature Backlog for registry-web + docs

Full feature roadmap to bring both web apps to production quality.
Based on audit of current state (2026-03-17).

---

## registry-web (`apps/registry-web/`) — Registry Dashboard & Site

### A — User Account & Auth

- [x] **Logout button in Header** — show user avatar/name + dropdown with "Logout" when session exists; hide Dashboard link when logged out
- [x] **User profile page** — `/account` — edit display name, change email, change password
- [x] **Email verification flow** — send verification email on register, block publishing until verified; `/auth/verify-email` page
- [x] **Password reset flow** — "Forgot password?" on login page → `/auth/reset-password` with email input and token-based confirmation page
- [x] **OAuth / social login** — Google + GitHub sign-in buttons on login/register pages (better-auth providers)
- [x] **Session management** — `/account/sessions` — list active sessions, revoke individual sessions
- [x] **Account deletion** — delete account button in settings with confirmation dialog

---

### B — Package Publishing (UI)

- [x] **Publish page** — `/_protected/publish` — multi-step form:
  - Step 1: Package name (`@tsx-pkg/<slug>`), version, description
  - Step 2: Upload `manifest.json` (textarea with JSON syntax validation) + tarball (.tar.gz file picker)
  - Step 3: Preview parsed metadata (lang, provides, integrates_with)
  - Step 4: Confirmation + submit → `POST /v1/packages/publish`
- [x] **Publish status / feedback** — show upload progress bar, success state with install command, error display with field-level validation messages
- [x] **API key management** — `/_protected/account/api-keys` — generate / revoke bearer tokens for CLI publishing (`tsx framework publish --api-key`); show key once on creation

---

### C — My Packages (Authenticated Users)

- [x] **My Packages page** — `/_protected/packages` — table of packages the logged-in user has published: name, latest version, downloads, last updated, actions (edit description, yank version, delete)
- [x] **Package edit page** — `/_protected/packages/:name/edit` — update description, add README (markdown textarea), manage versions (yank / set as latest)
- [x] **Package README upload/edit** — textarea that saves a markdown README into the registry server (requires new `PUT /v1/packages/:name` endpoint on the backend)

---

### D — Package Detail Enhancements

- [x] **README rendering** — fetch README from tarball on the package detail page and render as HTML with a markdown parser (e.g. `marked` or `micromark`); tabbed layout: Overview (README) / Versions / Install
- [x] **Syntax highlighting in README** — use `highlight.js` or `shiki` for fenced code blocks in rendered markdown
- [x] **Install command copy button** — clipboard copy on the `tsx registry install <pkg>` pill already shown; add visual feedback (checkmark icon)
- [x] **Provides & integrates_with badges** — render `provides[]` as coloured pill badges on the package card and detail page; render `integrates_with` as linked badges to the relevant package pages
- [x] **Dependency graph / integration map** — SVG hub-spoke diagram on package detail "Graph" tab; nodes are clickable and navigate to the related package
- [x] **Download trend chart** — bar chart for weekly downloads on package detail page (Downloads tab); uses proportional estimate, ready for real backend endpoint
- [x] **Version diff / changelog** — timeline version history with delta downloads between versions in the Versions tab
- [x] **Search by provides** — click a `provides` tag → browse all packages that provide the same capability

---

### E — Browse / Search Enhancements

- [x] **Pagination on browse** — replace current all-at-once load with page-based or infinite scroll; URL-synced `?q=` and `?page=` params
- [x] **Filter panel** — sidebar filters on `/browse`: by `lang` (TypeScript / Python / Rust / Go), by `runtime` (Node / Bun / Deno), by `provides` tag
- [x] **Sort options** — toggle between Most Downloaded, Newest, Recently Updated, Name A–Z
- [x] **Package cards** — add `lang` icon badges, verified/official badge for `@tsx-pkg/*` packages
- [x] **Empty state** — friendly illustration + "No packages found" message when search returns 0 results

---

### F — Landing Page Enhancements

- [x] **Hero install command** — animated terminal showing `cargo install tsx` → `tsx registry install tanstack-start` with blinking cursor
- [x] **Stats counters** — animate the numbers (total packages, downloads, versions) counting up on scroll-into-view
- [x] **Featured packages section** — curated grid of 6 official `@tsx-pkg/*` packages with icons
- [x] **"How it works" section** — 3-step visual: 1 Install CLI → 2 Add a package → 3 Run a generator
- [x] **Examples gallery section** — link to `examples/` with screenshots or code previews
- [x] **"Built with tsx" / showcase** — community-submitted projects using tsx packages

---

### G — Admin Dashboard (Role-Gated)

- [x] **Role guard middleware** — `/_protected/admin` — only accessible if `user.role === "admin"`; 403 page otherwise
- [x] **Admin overview** — all-time stats, charts for daily publishes/downloads (Recharts already installed)
- [x] **Package moderation** — table of all packages with Yank / Delete / Feature actions
- [x] **User management** — table of all users; promote to admin, suspend account
- [x] **Publish audit log** — `/admin/audit-log` table showing who published what and when; full IP log requires backend endpoint
- [x] **Rate limit monitor** — `/admin/rate-limits` IP rate monitor with block/unblock actions; live data requires backend endpoint

---

### H — UX / DX Improvements

- [x] **Mobile nav** — hamburger menu in Header with slide-out drawer (Sheet component already available) for small screens
- [x] **Global error boundary** — catch unhandled errors and show a friendly error page with retry button
- [x] **404 page** — custom `not-found.tsx` with suggested links
- [x] **Loading skeletons** — package card skeleton on browse, stat card skeleton on landing and dashboard
- [x] **Toast notifications** — use `sonner` (already installed) for publish success/fail, copy-to-clipboard, auth events
- [x] **Keyboard shortcuts** — `/` focuses search bar on browse page; `Escape` clears search
- [x] **SEO / meta tags** — `<title>`, `<meta description>`, Open Graph tags per route (TanStack Start `<Head>`)
- [x] **Sitemap + robots.txt** — generate `/sitemap.xml` listing all package pages for search engine indexing
- [x] **PWA manifest** — `manifest.json` with tsx registry branding, theme_color, start_url; linked in `__root.tsx` head

---

## docs (`apps/docs/`) — Documentation Site

### I — Missing Content Pages (Sidebar is ~20% complete)

> 12 of 15 sidebar routes have no page file. All need to be created with real content.

**Introduction**

- [x] **`/docs/installation`** — detailed install guide:
  - `cargo install tsx` (from crates.io)
  - `cargo install --git <repo> tsx` (from source)
  - Pre-built binaries from GitHub Releases
  - Windows / macOS / Linux specific notes
  - Shell completions (`tsx completions bash/zsh/fish/powershell`)
  - Verifying install: `tsx --version`

**CLI — Individual Command Pages**

- [x] **`/docs/cli/install`** — `tsx registry install <package>` — flags (`--registry`, `--dir`, `--force`), what gets written to `.tsx/packages/`, version pinning, offline mode
- [x] **`/docs/cli/search`** — `tsx registry search <query>` — `--lang`, `--json` flags, interpreting results, npm fallback
- [x] **`/docs/cli/info`** — `tsx registry info <package>` — full metadata output, `--json` flag, reading provides/integrates_with
- [x] **`/docs/cli/framework`** — `tsx framework init/validate/preview/add/publish` — full authoring workflow; links to FPF spec
- [x] **`/docs/cli/stack`** — `tsx stack init/show/add/remove/detect` — stack.json explained; `--install` auto-install flag; path aliases

**Framework Package Format (FPF)**

- [x] **`/docs/fpf`** — FPF overview — what is a framework package, directory layout (`.tsx/packages/<slug>/`), relationship between manifest + generators + templates
- [x] **`/docs/fpf/manifest`** — `stack.json` + `manifest.json` full field reference:
  - Top-level: `id`, `name`, `description`, `version`, `tsx_min`, `lang`, `runtime`
  - `provides[]` — capability tokens
  - `integrates_with{}` — slot injection map
  - `generators[]` — `id`, `command`, `description`, `schema`, `output_paths`
  - `style{}` — `quotes`, `indent`, `semicolons`
  - `paths{}` — path alias map
  - Full annotated JSON example
- [x] **`/docs/fpf/publishing`** — end-to-end guide: write manifest → write forge templates → `tsx framework validate` → `tsx framework preview` → `tsx framework publish --registry <url> --api-key <key>`

**Registry**

- [x] **`/docs/registry/self-hosting`** — running the registry server:
  - Binary vs Docker (`crates/registry-server/Dockerfile`)
  - Fly.io deployment with `fly.toml`
  - Env vars (`PORT`, `DATA_DIR`, `TSX_REGISTRY_API_KEY`)
  - Pointing the CLI: `TSX_REGISTRY_URL=https://...`
  - Backup strategy for SQLite WAL + tarballs volume
- [x] **`/docs/registry/api`** — full REST API reference with request/response examples for every endpoint:
  - `GET /health`
  - `GET /v1/stats`
  - `GET /v1/search?q=&lang=&size=`
  - `GET /v1/packages?sort=recent&limit=`
  - `GET /v1/packages/:name`
  - `GET /v1/packages/:name/versions`
  - `GET /v1/packages/:name/:version/tarball`
  - `POST /v1/packages/publish` (multipart fields, auth header, error codes)

---

### J — Docs DX & UX Improvements

- [x] **Syntax highlighting** — integrate `shiki` (or `highlight.js`) for all `<pre><code>` blocks; support `bash`, `json`, `typescript`, `rust`, `toml` languages
- [x] **Copy button on code blocks** — clipboard icon that appears on hover over code snippets
- [x] **Mobile sidebar** — hamburger button in Header opens a Sheet/drawer with the sidebar nav; close on link click
- [x] **Table of contents (ToC)** — auto-generated sticky right-side ToC for long pages (scan `h2`/`h3` headings); highlight active heading on scroll
- [x] **Breadcrumb navigation** — `Docs > CLI > tsx install` breadcrumb trail above page heading
- [x] **"Edit on GitHub" link** — per-page link to the source file in the repo
- [x] **Prev / Next page navigation** — footer navigation between adjacent sidebar pages
- [x] **Doc search** — integrate `pagefind` or `fuse.js` full-text search across all doc pages; `Cmd+K` opens command palette
- [ ] **Versioned docs** — version selector in Header (once CLI has multiple major versions) *(deferred — requires CLI v2 with a separate docs branch)*
- [x] **MDX support** — migrate from raw JSX content to `.mdx` files for easier content editing (Vite MDX plugin); keep React components available in MDX
- [x] **Dark/light code theme** — switch shiki theme alongside the app theme toggle (e.g. github-light / github-dark)
- [x] **`/docs/examples`** — Examples gallery page — card grid linking to `examples/basic-crud`, `examples/with-auth`, `examples/with-shadcn`, `examples/full-saas` with description + tech stack tags
- [x] **`/docs/packages`** — First-party packages reference page — table of all `@tsx-pkg/*` packages with provides[], install command, link to registry-web detail page
- [x] **Troubleshooting page** — `tsx registry install` fails (network, version mismatch), `tsx run` unknown command, stack detection issues

---

### K — Shared / Infra

- [x] **Shared UI package** — extract ThemeToggle into `packages/ui/` workspace package so registry-web and docs don't duplicate it; root `package.json` with Bun workspaces
- [x] **E2E tests** — Playwright tests for registry-web critical paths: landing, search, auth, navigation/404 (`e2e/*.spec.ts`); `bun test:e2e` + CI job
- [x] **Storybook or component playground** — document the UI component library in registry-web
- [x] **Lighthouse CI** — `.lighthouserc.json` + dedicated `lighthouse` CI job; asserts accessibility ≥ 0.85, warns on perf/SEO/best-practices
- [x] **Environment variable validation** — `zod` parse of `import.meta.env` at startup so missing vars fail loudly at build time

---

## Priority Order

| Priority | Item |
|----------|------|
| P0 — Blocking UX | Logout button (A), Mobile nav (H), 404 page (H), Error boundary (H) |
| P0 — Core feature | README rendering (D), Publish page (B), Missing doc pages (I) |
| P1 — High value | Syntax highlighting in docs (J), Copy button on code (J), Pagination (E), Filters (E) |
| P1 — High value | ToC + prev/next in docs (J), Mobile sidebar in docs (J) |
| P2 — Nice to have | Download chart (D), OAuth (A), Admin dashboard (G), Doc search (J) |
| P3 — Polish | Examples gallery (F/K), Animated hero (F), Shared UI package (K), E2E tests (K) |
