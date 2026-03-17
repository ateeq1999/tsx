import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/cli/framework.mdx"

export const Route = createFileRoute("/docs/cli/framework")({
  component: Content,
})
