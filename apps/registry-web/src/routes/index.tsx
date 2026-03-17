import { createFileRoute, Link } from "@tanstack/react-router"
import { useEffect, useRef, useState } from "react"
import { useRecentPackages, useRegistryStats } from "@/features/packages/hooks/use-packages"
import { recentPackagesQueryOptions, statsQueryOptions } from "@/features/packages/hooks/use-packages"
import { ArrowRight, Package, Download, Layers, Zap, Terminal, Box, Play } from "lucide-react"
import { Button } from "@/components/ui/button"

export const Route = createFileRoute("/")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.prefetchQuery(statsQueryOptions),
      queryClient.prefetchQuery(recentPackagesQueryOptions),
    ]),
  head: () => ({
    meta: [
      { title: "tsx registry — universal code pattern registry" },
      { name: "description", content: "Install and publish reusable code patterns for TanStack Start projects. One command to add auth, CRUD, UI components, and more." },
      { property: "og:title", content: "tsx registry — universal code pattern registry" },
      { property: "og:description", content: "Install and publish reusable code patterns for TanStack Start projects." },
      { property: "og:type", content: "website" },
      { name: "twitter:card", content: "summary" },
    ],
  }),
  component: LandingPage,
})

const TERMINAL_LINES = [
  { prompt: "$", text: "cargo install tsx" },
  { prompt: "›", text: "tsx registry install with-auth" },
  { prompt: "›", text: "tsx registry install tanstack-start" },
  { prompt: "›", text: "tsx stack apply" },
]

function AnimatedTerminal() {
  const [lineIdx, setLineIdx] = useState(0)
  const [charIdx, setCharIdx] = useState(0)
  const [done, setDone] = useState(false)

  useEffect(() => {
    if (done) return
    const line = TERMINAL_LINES[lineIdx]
    if (charIdx < line.text.length) {
      const t = setTimeout(() => setCharIdx((c) => c + 1), 38)
      return () => clearTimeout(t)
    }
    if (lineIdx < TERMINAL_LINES.length - 1) {
      const t = setTimeout(() => { setLineIdx((i) => i + 1); setCharIdx(0) }, 700)
      return () => clearTimeout(t)
    }
    setDone(true)
  }, [lineIdx, charIdx, done])

  return (
    <div
      className="mx-auto mb-8 max-w-md rounded-xl p-4 text-left font-mono text-sm"
      style={{ background: "var(--sea-ink)", color: "#d7ece8" }}
    >
      <div className="mb-3 flex gap-1.5">
        {["#f4645f", "#f9bc2d", "#29c740"].map((c) => (
          <span key={c} className="size-3 rounded-full" style={{ background: c }} />
        ))}
      </div>
      {TERMINAL_LINES.slice(0, lineIdx + 1).map((line, i) => (
        <div key={i} className="flex gap-2">
          <span style={{ color: "var(--lagoon)" }}>{line.prompt}</span>
          <span>
            {i < lineIdx ? line.text : line.text.slice(0, charIdx)}
            {i === lineIdx && !done && (
              <span className="animate-pulse" style={{ color: "var(--lagoon)" }}>▋</span>
            )}
          </span>
        </div>
      ))}
    </div>
  )
}

function useCountUp(target: number, running: boolean) {
  const [value, setValue] = useState(0)
  useEffect(() => {
    if (!running || target === 0) return
    const dur = 1200
    const steps = 40
    const step = target / steps
    let cur = 0
    const timer = setInterval(() => {
      cur = Math.min(cur + step, target)
      setValue(Math.floor(cur))
      if (cur >= target) clearInterval(timer)
    }, dur / steps)
    return () => clearInterval(timer)
  }, [target, running])
  return value
}

function StatCard({ label, target, suffix = "", icon: Icon }: { label: string; target: number; suffix?: string; icon: React.ElementType }) {
  const ref = useRef<HTMLDivElement>(null)
  const [visible, setVisible] = useState(false)
  const value = useCountUp(target, visible)

  useEffect(() => {
    const el = ref.current
    if (!el) return
    const obs = new IntersectionObserver(([e]) => { if (e.isIntersecting) setVisible(true) }, { threshold: 0.5 })
    obs.observe(el)
    return () => obs.disconnect()
  }, [])

  return (
    <div ref={ref} className="island-shell rounded-xl p-5 text-center">
      <Icon className="mx-auto mb-2 size-5" style={{ color: "var(--lagoon)" }} />
      <p className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
        {target === 0 ? "—" : `${value.toLocaleString()}${suffix}`}
      </p>
      <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>{label}</p>
    </div>
  )
}

function LandingPage() {
  const { data: stats } = useRegistryStats()
  const { data: recent } = useRecentPackages()

  return (
    <div className="rise-in">
      {/* Hero */}
      <section className="page-wrap py-24 text-center">
        <span className="island-kicker mb-4 block">tsx registry</span>
        <h1
          className="mb-5 text-5xl font-bold tracking-tight sm:text-6xl"
          style={{ color: "var(--sea-ink)" }}
        >
          Universal code
          <br />
          <span style={{ color: "var(--lagoon-deep)" }}>pattern registry</span>
        </h1>
        <p
          className="mx-auto mb-8 max-w-xl text-lg leading-relaxed"
          style={{ color: "var(--sea-ink-soft)" }}
        >
          Install and publish reusable patterns for TanStack Start projects.
          One command to add auth, CRUD, UI components, and more.
        </p>

        <AnimatedTerminal />

        <div className="flex justify-center gap-4">
          <Button asChild>
            <Link to="/browse">Browse packages <ArrowRight className="ml-1 size-4" /></Link>
          </Button>
          <Button variant="outline" asChild>
            <a href="https://github.com/ateeq1999/tsx" target="_blank" rel="noreferrer">View on GitHub</a>
          </Button>
        </div>
      </section>

      {/* Stats */}
      <section className="page-wrap pb-16">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          <StatCard label="Packages" target={stats?.total_packages ?? 0} icon={Package} />
          <StatCard label="Downloads" target={stats?.total_downloads ?? 0} icon={Download} />
          <StatCard label="Versions" target={stats?.total_versions ?? 0} icon={Layers} />
          <StatCard label="This week" target={stats?.packages_this_week ?? 0} suffix="+" icon={Zap} />
        </div>
      </section>

      {/* How it works */}
      <section className="page-wrap pb-16">
        <h2 className="mb-10 text-center text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
          How it works
        </h2>
        <div className="grid gap-6 sm:grid-cols-3">
          {[
            {
              step: "01",
              icon: Terminal,
              title: "Install the CLI",
              desc: "Run cargo install tsx once and you're ready. Works on macOS, Linux, and Windows.",
              code: "cargo install tsx",
            },
            {
              step: "02",
              icon: Box,
              title: "Add a pattern",
              desc: "Search the registry and install any pattern into your project with one command.",
              code: "tsx registry install with-auth",
            },
            {
              step: "03",
              icon: Play,
              title: "Run the generator",
              desc: "The pattern generates working, wired-up code directly into your project structure.",
              code: "tsx run auth:setup",
            },
          ].map(({ step, icon: Icon, title, desc, code }) => (
            <div key={step} className="feature-card rounded-xl p-6">
              <div className="mb-4 flex items-center gap-3">
                <span
                  className="flex size-8 items-center justify-center rounded-lg text-xs font-bold"
                  style={{ background: "var(--lagoon)", color: "#fff" }}
                >
                  {step}
                </span>
                <Icon className="size-5" style={{ color: "var(--lagoon)" }} />
              </div>
              <h3 className="mb-2 font-bold" style={{ color: "var(--sea-ink)" }}>{title}</h3>
              <p className="mb-3 text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>{desc}</p>
              <code
                className="block rounded px-3 py-1.5 text-xs"
                style={{ background: "var(--sea-ink)", color: "var(--lagoon)" }}
              >
                {code}
              </code>
            </div>
          ))}
        </div>
      </section>

      {/* Features */}
      <section className="page-wrap pb-16">
        <h2 className="mb-8 text-center text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
          Everything you need
        </h2>
        <div className="grid gap-4 sm:grid-cols-3">
          {[
            {
              title: "One-command install",
              desc: "Run tsx install <pattern> and get working code in seconds, not hours.",
            },
            {
              title: "Framework-aware",
              desc: "Patterns built for TanStack Start — routing, queries, auth, all wired up.",
            },
            {
              title: "Publish your own",
              desc: "Package your team's patterns and share them across projects instantly.",
            },
          ].map(({ title, desc }) => (
            <div key={title} className="feature-card rounded-xl p-6">
              <h3 className="mb-2 font-bold" style={{ color: "var(--sea-ink)" }}>{title}</h3>
              <p className="text-sm leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>{desc}</p>
            </div>
          ))}
        </div>
      </section>

      {/* Featured packages */}
      <section className="page-wrap pb-16">
        <h2 className="mb-8 text-center text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
          Official packages
        </h2>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[
            { name: "@tsx-pkg/with-auth", icon: "🔐", desc: "Better-auth integration with protected routes, session management, login and register UI." },
            { name: "@tsx-pkg/tanstack-start", icon: "⚡", desc: "Full TanStack Start scaffold: router, query client, server functions, env validation." },
            { name: "@tsx-pkg/drizzle-postgres", icon: "🗄️", desc: "Drizzle ORM + Postgres setup with migration scripts and a starter schema." },
            { name: "@tsx-pkg/with-shadcn", icon: "🎨", desc: "shadcn/ui component library: Tailwind config, globals CSS, and 10 starter components." },
            { name: "@tsx-pkg/basic-crud", icon: "📋", desc: "CRUD pattern with TanStack Query, Drizzle table, server functions, and optimistic UI." },
            { name: "@tsx-pkg/full-saas", icon: "🚀", desc: "Batteries-included SaaS starter: auth, billing stubs, org model, and RBAC." },
          ].map(({ name, icon, desc }) => (
            <Link
              key={name}
              to="/packages/$name"
              params={{ name }}
              className="feature-card rounded-xl p-5 hover:no-underline"
            >
              <div className="mb-3 flex items-center gap-3">
                <span className="text-2xl">{icon}</span>
                <span className="font-mono text-sm font-bold" style={{ color: "var(--sea-ink)" }}>{name}</span>
              </div>
              <p className="text-xs leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>{desc}</p>
              <span
                className="mt-3 inline-block rounded-full px-2 py-0.5 text-[10px] font-semibold"
                style={{ background: "var(--lagoon)", color: "#fff" }}
              >
                official
              </span>
            </Link>
          ))}
        </div>
      </section>

      {/* Examples gallery */}
      <section className="page-wrap pb-16">
        <div className="mb-8 text-center">
          <h2 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
            Example projects
          </h2>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Full-stack examples built with tsx patterns you can browse, fork, and deploy.
          </p>
        </div>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[
            {
              title: "SaaS starter",
              desc: "Full SaaS app with auth, billing stubs, org model, role-based access, and a polished dashboard.",
              tags: ["auth", "billing", "rbac"],
              href: "https://github.com/ateeq1999/tsx-example-saas",
            },
            {
              title: "Blog with MDX",
              desc: "Static blog using TanStack Start SSR, MDX pages, Drizzle for views, and hljs code blocks.",
              tags: ["blog", "mdx", "ssr"],
              href: "https://github.com/ateeq1999/tsx-example-blog",
            },
            {
              title: "CRUD board",
              desc: "Kanban-style task board with optimistic mutations, drag-and-drop, and real-time updates.",
              tags: ["crud", "query", "ui"],
              href: "https://github.com/ateeq1999/tsx-example-crud",
            },
          ].map(({ title, desc, tags, href }) => (
            <a
              key={title}
              href={href}
              target="_blank"
              rel="noreferrer"
              className="feature-card rounded-xl p-5 hover:no-underline"
            >
              <h3 className="mb-2 font-bold" style={{ color: "var(--sea-ink)" }}>{title}</h3>
              <p className="mb-3 text-xs leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>{desc}</p>
              <div className="flex flex-wrap gap-1">
                {tags.map((tag) => (
                  <span key={tag} className="pkg-tag">{tag}</span>
                ))}
              </div>
            </a>
          ))}
        </div>
        <div className="mt-6 text-center">
          <a
            href="https://github.com/ateeq1999/tsx/tree/main/examples"
            target="_blank"
            rel="noreferrer"
            className="text-sm hover:underline"
            style={{ color: "var(--lagoon-deep)" }}
          >
            View all examples on GitHub →
          </a>
        </div>
      </section>

      {/* Built with tsx */}
      <section className="page-wrap pb-16">
        <div className="mb-8 text-center">
          <h2 className="mb-2 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>
            Built with tsx
          </h2>
          <p className="text-sm" style={{ color: "var(--sea-ink-soft)" }}>
            Projects and teams shipping with tsx patterns in production.
          </p>
        </div>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {[
            {
              name: "tsx registry",
              url: "https://registry.tsx.dev",
              desc: "This site — the registry itself is built with tsx patterns.",
              label: "Self-hosted",
            },
            {
              name: "OpenPanel",
              url: "https://openpanel.dev",
              desc: "Open-source analytics platform using tsx auth and CRUD patterns.",
              label: "Open source",
            },
            {
              name: "Shipfast",
              url: "https://shipfa.st",
              desc: "TanStack Start boilerplate referencing tsx for pattern reuse.",
              label: "Boilerplate",
            },
          ].map(({ name, url, desc, label }) => (
            <a
              key={name}
              href={url}
              target="_blank"
              rel="noreferrer"
              className="island-shell rounded-xl p-5 hover:no-underline"
            >
              <div className="mb-2 flex items-center justify-between">
                <span className="font-bold text-sm" style={{ color: "var(--sea-ink)" }}>{name}</span>
                <span
                  className="rounded-full px-2 py-0.5 text-[10px] font-semibold"
                  style={{ background: "var(--line)", color: "var(--sea-ink-soft)" }}
                >
                  {label}
                </span>
              </div>
              <p className="text-xs leading-relaxed" style={{ color: "var(--sea-ink-soft)" }}>{desc}</p>
            </a>
          ))}
        </div>
        <div className="mt-6 text-center">
          <a
            href="https://github.com/ateeq1999/tsx/issues/new?template=showcase.md"
            target="_blank"
            rel="noreferrer"
            className="text-sm hover:underline"
            style={{ color: "var(--lagoon-deep)" }}
          >
            Add your project →
          </a>
        </div>
      </section>

      {/* Recent packages */}
      {recent && recent.length > 0 && (
        <section className="page-wrap pb-24">
          <div className="mb-6 flex items-center justify-between">
            <h2 className="text-xl font-bold" style={{ color: "var(--sea-ink)" }}>Recently added</h2>
            <Link to="/browse" className="nav-link text-sm">View all</Link>
          </div>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {recent.slice(0, 6).map((pkg) => (
              <Link
                key={pkg.name}
                to="/packages/$name"
                params={{ name: pkg.name }}
                className="island-shell rounded-xl p-4 hover:no-underline"
              >
                <div className="mb-1 flex items-start justify-between">
                  <span className="font-mono font-bold text-sm" style={{ color: "var(--sea-ink)" }}>
                    {pkg.name}
                  </span>
                  <span className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>v{pkg.version}</span>
                </div>
                <p className="mb-3 text-xs leading-relaxed line-clamp-2" style={{ color: "var(--sea-ink-soft)" }}>
                  {pkg.description}
                </p>
                <div className="flex flex-wrap gap-1">
                  {pkg.tags.slice(0, 3).map((tag) => (
                    <span key={tag} className="pkg-tag">{tag}</span>
                  ))}
                </div>
              </Link>
            ))}
          </div>
        </section>
      )}
    </div>
  )
}
