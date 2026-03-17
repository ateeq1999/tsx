import { z } from "zod"

const envSchema = z.object({
  VITE_REGISTRY_URL: z.string().url().optional().default("http://localhost:8080"),
  VITE_SITE_URL: z.string().url().optional().default("https://registry.tsx.dev"),
  VITE_SERVER_URL: z.string().url().optional().default("http://localhost:3000"),
  VITE_CORS_ORIGIN: z.string().optional().default("http://localhost:3000"),
})

function validateEnv() {
  const result = envSchema.safeParse({
    VITE_REGISTRY_URL: import.meta.env.VITE_REGISTRY_URL,
    VITE_SITE_URL: import.meta.env.VITE_SITE_URL,
    VITE_SERVER_URL: import.meta.env.VITE_SERVER_URL,
    VITE_CORS_ORIGIN: import.meta.env.VITE_CORS_ORIGIN,
  })

  if (!result.success) {
    const issues = result.error.issues
      .map((i) => `  ${i.path.join(".")}: ${i.message}`)
      .join("\n")
    throw new Error(`Invalid environment variables:\n${issues}`)
  }

  return result.data
}

export const env = validateEnv()
