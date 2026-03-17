import { defineConfig } from "vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact from "@vitejs/plugin-react"
import viteTsConfigPaths from "vite-tsconfig-paths"
import tailwindcss from "@tailwindcss/vite"
import { nitro } from "nitro/vite"
import mdx from "@mdx-js/rollup"
import remarkGfm from "remark-gfm"
import rehypeSlug from "rehype-slug"

export default defineConfig({
  plugins: [
    // MDX must come before the React plugin
    { enforce: "pre", ...mdx({ remarkPlugins: [remarkGfm], rehypePlugins: [rehypeSlug] }) },
    nitro(),
    viteTsConfigPaths({ projects: ["./tsconfig.json"] }),
    tailwindcss(),
    tanstackStart(),
    viteReact({ include: /\.(jsx|tsx)$/ }),
  ],
})
