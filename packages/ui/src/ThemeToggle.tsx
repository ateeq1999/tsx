import type { ComponentPropsWithoutRef } from "react"
import { useTheme } from "next-themes"
import { Moon, Sun } from "lucide-react"

type ThemeToggleProps = Omit<ComponentPropsWithoutRef<"button">, "onClick" | "aria-label" | "children">

export function ThemeToggle(props: ThemeToggleProps) {
  const { resolvedTheme, setTheme } = useTheme()
  return (
    <button
      {...props}
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
      aria-label="Toggle theme"
    >
      {resolvedTheme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
    </button>
  )
}
