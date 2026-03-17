import type React from "react"
import { ThemeToggle as BaseThemeToggle } from "@tsx/ui/theme-toggle"

export function ThemeToggle() {
  return (
    <BaseThemeToggle
      className="rounded p-1 transition-colors hover:opacity-80"
      style={{ color: "var(--sea-ink-soft)" } as React.CSSProperties}
    />
  )
}
