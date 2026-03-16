import { createFileRoute, Link, Outlet } from "@tanstack/react-router"

export const Route = createFileRoute("/docs")({
  component: DocsLayout,
})

const sidebar = [
  {
    group: "Introduction",
    links: [
      { to: "/docs/getting-started", label: "Getting Started" },
      { to: "/docs/installation", label: "Installation" },
    ],
  },
  {
    group: "CLI",
    links: [
      { to: "/docs/cli", label: "Overview" },
      { to: "/docs/cli/install", label: "tsx install" },
      { to: "/docs/cli/search", label: "tsx search" },
      { to: "/docs/cli/info", label: "tsx info" },
      { to: "/docs/cli/framework", label: "tsx framework" },
      { to: "/docs/cli/stack", label: "tsx stack" },
    ],
  },
  {
    group: "Framework Packages",
    links: [
      { to: "/docs/fpf", label: "FPF Format" },
      { to: "/docs/fpf/manifest", label: "stack.json" },
      { to: "/docs/fpf/publishing", label: "Publishing" },
    ],
  },
  {
    group: "Registry",
    links: [
      { to: "/docs/registry", label: "Overview" },
      { to: "/docs/registry/self-hosting", label: "Self-hosting" },
      { to: "/docs/registry/api", label: "API Reference" },
    ],
  },
]

function DocsLayout() {
  return (
    <div className="page-wrap py-10">
      <div className="flex gap-10">
        {/* Sidebar */}
        <aside className="hidden w-52 shrink-0 lg:block">
          <nav className="sticky top-20 space-y-6">
            {sidebar.map((section) => (
              <div key={section.group}>
                <p className="island-kicker mb-2">{section.group}</p>
                <ul className="space-y-1">
                  {section.links.map((link) => (
                    <li key={link.to}>
                      <Link
                        to={link.to}
                        className="nav-link block py-1 text-sm"
                        activeProps={{ className: "nav-link is-active block py-1 text-sm font-semibold" }}
                      >
                        {link.label}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </nav>
        </aside>

        {/* Content */}
        <article className="doc-content min-w-0 flex-1">
          <Outlet />
        </article>
      </div>
    </div>
  )
}
