import { createFileRoute, Link } from "@tanstack/react-router"
import { useRecentPackages, useRegistryStats } from "@/features/packages/hooks/use-packages"
import { recentPackagesQueryOptions, statsQueryOptions } from "@/features/packages/hooks/use-packages"
import { ArrowRight, Package, Download, Layers, Zap } from "lucide-react"
import { Button } from "@/components/ui/button"

export const Route = createFileRoute("/")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.prefetchQuery(statsQueryOptions),
      queryClient.prefetchQuery(recentPackagesQueryOptions),
    ]),
  component: LandingPage,
})

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

        {/* Install command */}
        <div
          className="island-shell mx-auto mb-8 inline-flex max-w-sm items-center gap-3 rounded-lg px-5 py-3"
        >
          <span style={{ color: "var(--lagoon)" }} className="text-sm font-bold select-none">$</span>
          <code className="flex-1 text-left text-sm" style={{ color: "var(--sea-ink)" }}>
            tsx install with-auth
          </code>
          <button
            onClick={() => navigator.clipboard.writeText("tsx install with-auth")}
            className="text-xs opacity-60 hover:opacity-100"
            style={{ color: "var(--lagoon-deep)" }}
          >
            copy
          </button>
        </div>

        <div className="flex justify-center gap-4">
          <Button asChild>
            <Link to="/browse">Browse packages <ArrowRight className="ml-1 size-4" /></Link>
          </Button>
          <Button variant="outline" asChild>
            <a href="https://github.com/your-org/tsx" target="_blank" rel="noreferrer">View on GitHub</a>
          </Button>
        </div>
      </section>

      {/* Stats */}
      {stats && (
        <section className="page-wrap pb-16">
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            {[
              { label: "Packages", value: stats.total_packages, icon: Package },
              { label: "Downloads", value: stats.total_downloads.toLocaleString(), icon: Download },
              { label: "Versions", value: stats.total_versions, icon: Layers },
              { label: "This week", value: `+${stats.packages_this_week}`, icon: Zap },
            ].map(({ label, value, icon: Icon }) => (
              <div key={label} className="island-shell rounded-xl p-5 text-center">
                <Icon className="mx-auto mb-2 size-5" style={{ color: "var(--lagoon)" }} />
                <p className="text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>{value}</p>
                <p className="text-xs" style={{ color: "var(--sea-ink-soft)" }}>{label}</p>
              </div>
            ))}
          </div>
        </section>
      )}

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
