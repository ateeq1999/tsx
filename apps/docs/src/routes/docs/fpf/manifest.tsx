import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/fpf/manifest.mdx"

export const Route = createFileRoute("/docs/fpf/manifest")({
  component: Content,
})
