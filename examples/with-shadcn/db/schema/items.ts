import { pgTable, serial, text, boolean, timestamp } from "drizzle-orm/pg-core"

export const itemsTable = pgTable("items", {
  id: serial("id").primaryKey(),
  title: text("title").notNull(),
  description: text("description"),
  completed: boolean("completed").default(false).notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
})

export type Item = typeof itemsTable.$inferSelect
export type NewItem = typeof itemsTable.$inferInsert
