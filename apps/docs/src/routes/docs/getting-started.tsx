import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/getting-started.mdx"

export const Route = createFileRoute("/docs/getting-started")({
  component: Content,
})
