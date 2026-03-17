import { createServerFn } from "@tanstack/react-start"
import { desc } from "drizzle-orm"
import { db } from "@/db"
import { user } from "@/db/schema/auth"
import { requireRole } from "@/middleware/role-guard"

export const getAdminUsers = createServerFn({ method: "GET" }).handler(async () => {
  await requireRole("admin")
  return db
    .select({
      id: user.id,
      name: user.name,
      email: user.email,
      emailVerified: user.emailVerified,
      createdAt: user.createdAt,
    })
    .from(user)
    .orderBy(desc(user.createdAt))
})
