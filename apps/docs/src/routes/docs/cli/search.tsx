import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/cli/search.mdx"

export const Route = createFileRoute("/docs/cli/search")({
  component: Content,
})
