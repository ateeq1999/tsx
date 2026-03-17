import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/cli/install.mdx"

export const Route = createFileRoute("/docs/cli/install")({
  component: Content,
})
