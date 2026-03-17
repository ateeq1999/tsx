import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/installation.mdx"

export const Route = createFileRoute("/docs/installation")({
  component: Content,
})
