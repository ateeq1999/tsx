import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/docs/examples")({
  component: ExamplesPage,
})

const examples = [
  {
    slug: "basic-crud",
    title: "Basic CRUD",
    desc: "Full create/read/update/delete pattern with TanStack Query, a Drizzle ORM table, and server functions. Includes an optimistic UI layer.",
    tags: ["typescript", "drizzle", "tanstack-query"],
    install: "tsx registry install basic-crud",
  },
  {
    slug: "with-auth",
    title: "With Auth",
    desc: "Better-auth integration: email/password sign-up, session management, protected routes, and a pre-built login/register UI.",
    tags: ["typescript", "better-auth", "tanstack-start"],
    install: "tsx registry install with-auth",
  },
  {
    slug: "with-shadcn",
    title: "With shadcn/ui",
    desc: "Bootstraps a shadcn/ui component library in your project: tailwind config, globals CSS, and 10 starter components.",
    tags: ["typescript", "tailwind", "shadcn"],
    install: "tsx registry install with-shadcn",
  },
  {
    slug: "tanstack-start",
    title: "TanStack Start Stack",
    desc: "Full TanStack Start scaffold: router, query client, server functions, environment validation, and recommended folder layout.",
    tags: ["typescript", "tanstack-start", "vite"],
    install: "tsx registry install tanstack-start",
  },
  {
    slug: "drizzle-postgres",
    title: "Drizzle + Postgres",
    desc: "Drizzle ORM setup with a Postgres connection, migration scripts, and a sample schema with created_at/updated_at helpers.",
    tags: ["typescript", "drizzle", "postgres"],
    install: "tsx registry install drizzle-postgres",
  },
  {
    slug: "full-saas",
    title: "Full SaaS Starter",
    desc: "Batteries-included SaaS starter: auth, billing stubs, team/org model, dashboard shell, and role-based access control.",
    tags: ["typescript", "saas", "better-auth", "drizzle"],
    install: "tsx registry install full-saas",
  },
]

const TAG_COLOURS: Record<string, string> = {
  typescript: "#3178c6",
  drizzle: "#c5f074",
  "tanstack-query": "#ff4154",
  "tanstack-start": "#ff4154",
  "better-auth": "#7c3aed",
  tailwind: "#38bdf8",
  shadcn: "#18181b",
  vite: "#646cff",
  postgres: "#336791",
  saas: "#f59e0b",
}

function TagPill({ tag }: { tag: string }) {
  const bg = TAG_COLOURS[tag] ?? "#416166"
  const dark = ["shadcn", "drizzle", "tailwind"].includes(tag)
  return (
    <span
      className="rounded px-1.5 py-0.5 text-[10px] font-semibold"
      style={{ background: bg, color: dark ? "#fff" : "#fff" }}
    >
      {tag}
    </span>
  )
}

function ExamplesPage() {
  return (
    <div>
      <h1>Examples</h1>
      <p>
        Browse community-maintained example packages. Each one can be installed directly into your TanStack Start project
        with a single <code>tsx registry install</code> command.
      </p>
      <p>
        Don't see what you need?{" "}
        <a href="https://github.com/ateeq1999/tsx" target="_blank" rel="noreferrer">Open a PR</a> to add your own package to the registry.
      </p>

      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        {examples.map((ex) => (
          <div
            key={ex.slug}
            className="rounded-xl border p-5"
            style={{ borderColor: "var(--line)", background: "var(--surface)" }}
          >
            <h3 style={{ marginTop: 0 }}>{ex.title}</h3>
            <p style={{ marginBottom: "0.75rem" }}>{ex.desc}</p>
            <div className="mb-3 flex flex-wrap gap-1">
              {ex.tags.map((t) => <TagPill key={t} tag={t} />)}
            </div>
            <pre><code className="language-bash">{ex.install}</code></pre>
          </div>
        ))}
      </div>

      <h2>Building your own</h2>
      <p>
        Any package on the registry can serve as an example. To publish your own pattern:
      </p>
      <ol>
        <li>Write a <code>manifest.json</code> describing your package</li>
        <li>Add generator templates in the <code>generators/</code> directory</li>
        <li>Run <code>tsx framework validate</code> to check your manifest</li>
        <li>Run <code>tsx framework publish</code> to upload to the registry</li>
      </ol>
      <p>
        See the <a href="/docs/fpf/publishing">Publishing guide</a> for the full walkthrough.
      </p>
    </div>
  )
}
