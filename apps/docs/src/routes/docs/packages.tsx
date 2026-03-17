import { createFileRoute } from "@tanstack/react-router"
import Content from "@/content/packages.mdx"

export const Route = createFileRoute("/docs/packages")({
  component: Content,
})
