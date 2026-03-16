import { createFileRoute, Link } from "@tanstack/react-router"
import { ArrowRight, Terminal, Package, Layers } from "lucide-react"

export const Route = createFileRoute("/")({
  component: DocsHome,
})

function DocsHome() {
  return (
    <div className="rise-in">
      {/* Hero */}
      <section className="page-wrap py-24 text-center">
        <span className="island-kicker mb-4 block">tsx documentation</span>
        <h1 className="mb-5 text-5xl font-bold tracking-tight" style={{ color: "var(--sea-ink)" }}>
          Build faster with
          <br />
          <span style={{ color: "var(--lagoon-deep)" }}>reusable patterns</span>
        </h1>
        <p className="mx-auto mb-10 max-w-lg text-lg leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>
          tsx is a universal code pattern registry for TanStack Start projects.
          Install auth, CRUD, UI patterns, and more with a single command.
        </p>
        <div className="flex justify-center gap-4">
          <Link
            to="/docs/getting-started"
            className="inline-flex items-center gap-2 rounded-lg px-6 py-3 text-sm font-semibold text-white transition-all hover:opacity-90"
            style={{ background: "var(--lagoon-deep)" }}
          >
            Get started <ArrowRight className="size-4" />
          </Link>
          <Link
            to="/docs/cli"
            className="inline-flex items-center gap-2 rounded-lg border px-6 py-3 text-sm font-semibold transition-all"
            style={{ borderColor: "var(--line)", color: "var(--sea-ink-soft)" }}
          >
            CLI Reference
          </Link>
        </div>
      </section>

      {/* Quick nav cards */}
      <section className="page-wrap pb-24">
        <div className="grid gap-4 sm:grid-cols-3">
          {[
            {
              to: "/docs/getting-started" as const,
              icon: Terminal,
              title: "Getting Started",
              desc: "Install tsx and run your first pattern in under 2 minutes.",
            },
            {
              to: "/docs/cli" as const,
              icon: Layers,
              title: "CLI Reference",
              desc: "All commands, flags, and configuration options documented.",
            },
            {
              to: "/docs/registry" as const,
              icon: Package,
              title: "Registry",
              desc: "Self-host a private registry or publish to the public one.",
            },
          ].map(({ to, icon: Icon, title, desc }) => (
            <Link
              key={to}
              to={to}
              className="feature-card rounded-xl p-6 no-underline hover:no-underline"
            >
              <Icon className="mb-3 size-6" style={{ color: "var(--lagoon)" }} />
              <h3 className="mb-1 font-bold" style={{ color: "var(--sea-ink)" }}>{title}</h3>
              <p className="text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>{desc}</p>
            </Link>
          ))}
        </div>
      </section>
    </div>
  )
}
