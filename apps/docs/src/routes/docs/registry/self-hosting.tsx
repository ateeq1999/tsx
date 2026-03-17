import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/registry/self-hosting.mdx"

export const Route = createFileRoute("/docs/registry/self-hosting")({
  component: Content,
})
