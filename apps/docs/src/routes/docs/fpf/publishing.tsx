import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/fpf/publishing.mdx"

export const Route = createFileRoute("/docs/fpf/publishing")({
  component: Content,
})
