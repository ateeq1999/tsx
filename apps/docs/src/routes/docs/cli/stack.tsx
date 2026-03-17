import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/cli/stack.mdx"

export const Route = createFileRoute("/docs/cli/stack")({
  component: Content,
})
