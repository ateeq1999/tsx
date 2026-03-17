import type { ReactNode } from "react"

interface BaseHeaderProps {
  logo?: ReactNode
  nav?: ReactNode
  right?: ReactNode
}

export function BaseHeader({ logo, nav, right }: BaseHeaderProps) {
  return (
    <header
      style={{ background: "var(--header-bg)", borderBottom: "1px solid var(--line)" }}
      className="sticky top-0 z-50 backdrop-blur-md"
    >
      <div className="page-wrap flex h-14 items-center justify-between gap-4">
        {logo}
        {nav}
        {right && <div className="flex items-center gap-2">{right}</div>}
      </div>
    </header>
  )
}
