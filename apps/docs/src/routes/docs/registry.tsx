import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/registry.mdx"

export const Route = createFileRoute("/docs/registry")({
  component: Content,
})
