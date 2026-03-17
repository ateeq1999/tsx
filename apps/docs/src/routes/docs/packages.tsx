import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/packages")({
  component: PackagesReferencePage,
})

const REGISTRY_URL = "https://registry.tsx.dev"

const packages = [
  {
    name: "@tsx-pkg/with-auth",
    provides: ["auth", "session", "protected-routes"],
    desc: "Better-auth email/password auth, Drizzle session table, login/register forms, protected route middleware.",
    lang: "typescript",
    runtime: "bun",
  },
  {
    name: "@tsx-pkg/tanstack-start",
    provides: ["router", "query-client", "server-functions", "env-validation"],
    desc: "Full TanStack Start scaffold — router, query client with SSR, server functions, and zod env schema.",
    lang: "typescript",
    runtime: "bun",
  },
  {
    name: "@tsx-pkg/drizzle-postgres",
    provides: ["database", "migrations", "schema"],
    desc: "Drizzle ORM + node-postgres connection, migration scripts, and a starter schema with helpers.",
    lang: "typescript",
    runtime: "node",
  },
  {
    name: "@tsx-pkg/with-shadcn",
    provides: ["ui-components", "tailwind", "design-system"],
    desc: "shadcn/ui component library bootstrap: tailwind.config, globals.css, and 10 base components.",
    lang: "typescript",
    runtime: "bun",
  },
  {
    name: "@tsx-pkg/basic-crud",
    provides: ["crud", "server-functions", "optimistic-ui"],
    desc: "Full CRUD pattern — Drizzle table, TanStack Query hooks, server functions, optimistic mutations.",
    lang: "typescript",
    runtime: "bun",
  },
  {
    name: "@tsx-pkg/full-saas",
    provides: ["auth", "billing", "teams", "rbac", "dashboard"],
    desc: "SaaS starter: auth, billing stubs, org/team model, role-based access control, admin shell.",
    lang: "typescript",
    runtime: "bun",
  },
]

function ProvidesBadge({ cap }: { cap: string }) {
  return (
    <span
      className="rounded px-1.5 py-0.5 text-[10px] font-semibold"
      style={{ background: "var(--lagoon)", color: "#fff" }}
    >
      {cap}
    </span>
  )
}

function PackagesReferencePage() {
  return (
    <div>
      <h1>First-party Packages</h1>
      <p>
        Official <code>@tsx-pkg/*</code> packages are maintained by the tsx project and published to the{" "}
        <a href={REGISTRY_URL} target="_blank" rel="noreferrer">public registry</a>. They follow the{" "}
        <a href="/docs/fpf">FPF format</a> and can be installed with one command.
      </p>
      <p>
        Each package declares a <code>provides[]</code> list of capability tokens. Other packages can declare
        <code>integrates_with</code> dependencies on these tokens, enabling automatic slot injection.
      </p>

      <h2>Package index</h2>

      <div className="not-prose mt-4 space-y-4">
        {packages.map((pkg) => (
          <div
            key={pkg.name}
            className="rounded-xl border p-5"
            style={{ borderColor: "var(--line)", background: "var(--surface)" }}
          >
            <div className="mb-2 flex flex-wrap items-center gap-2">
              <code style={{ fontSize: "0.9rem", fontWeight: 700, color: "var(--sea-ink)" }}>{pkg.name}</code>
              <span
                className="rounded px-1.5 py-0.5 text-[10px] font-semibold"
                style={{ background: "var(--lagoon)", color: "#fff" }}
              >
                official
              </span>
              <span className="text-xs capitalize" style={{ color: "var(--sea-ink-soft)" }}>{pkg.lang} · {pkg.runtime}</span>
            </div>
            <p style={{ color: "var(--sea-ink-soft)", marginBottom: "0.75rem", fontSize: "0.875rem" }}>{pkg.desc}</p>
            <div className="mb-3 flex flex-wrap gap-1">
              {pkg.provides.map((cap) => <ProvidesBadge key={cap} cap={cap} />)}
            </div>
            <pre style={{ marginBottom: 0 }}><code className="language-bash">{`tsx registry install ${pkg.name}`}</code></pre>
          </div>
        ))}
      </div>

      <h2>Publishing your own</h2>
      <p>
        Anyone can publish packages to the registry. See the{" "}
        <a href="/docs/fpf/publishing">Publishing guide</a> to get started.
        Community packages appear alongside first-party packages in search results.
      </p>

      <h2>Providing capability tokens</h2>
      <p>
        The <code>provides[]</code> field in <code>manifest.json</code> lists abstract capability tokens
        your package fulfills. For example, a package that sets up authentication should list <code>"auth"</code>.
        Other packages can then declare <code>integrates_with: {"{ \"auth\": \"...\" }"}  </code> to inject
        code into your package's slots.
      </p>
      <p>
        Standardised token names ensure interoperability — use the tokens from existing official packages
        where possible, or introduce new ones with clear documentation.
      </p>
    </div>
  )
}
