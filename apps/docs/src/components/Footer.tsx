import { Footer as BaseFooter } from "@tsx/ui/footer"

const LINKS = [
  { label: "Docs", href: "/docs/getting-started" },
  { label: "GitHub", href: "https://github.com/ateeq1999/tsx", external: true },
]

export function Footer() {
  return (
    <BaseFooter
      appName="tsx — universal code pattern registry"
      links={LINKS}
    />
  )
}
