import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/registry/api.mdx"

export const Route = createFileRoute("/docs/registry/api")({
  component: Content,
})
