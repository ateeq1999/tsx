import type { ReactNode } from "react"

interface FooterLink {
  label: string
  href: string
  external?: boolean
}

interface FooterProps {
  appName?: string
  links?: FooterLink[]
  children?: ReactNode
}

export function Footer({ appName = "tsx", links = [], children }: FooterProps) {
  return (
    <footer className="site-footer mt-auto py-8">
      <div className="page-wrap flex flex-col items-center justify-between gap-4 text-sm sm:flex-row">
        <p style={{ color: "var(--sea-ink-soft)" }}>
          &copy; {new Date().getFullYear()} {appName}
        </p>
        {links.length > 0 && (
          <div className="flex gap-6" style={{ color: "var(--sea-ink-soft)" }}>
            {links.map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="nav-link"
                {...(link.external ? { target: "_blank", rel: "noreferrer" } : {})}
              >
                {link.label}
              </a>
            ))}
          </div>
        )}
        {children}
      </div>
    </footer>
  )
}
