import { createServerFn } from "@tanstack/start/server";
import { db } from "~/db";
import { productsTable } from "~/db/schema/products";
import { eq } from "drizzle-orm";
import { z } from "zod";

const ProductInputSchema = z.object({
  title: z.string().min(1),
  description: z.string().optional(),
  price: z.number().int().positive(),
  in_stock: z.boolean().optional(),
});

const ProductIdSchema = z.object({ id: z.number() });

export const listProducts = createServerFn({ method: "GET" })
  .handler(async () => {
    return db.select().from(productsTable);
  });

export const getProduct = createServerFn({ method: "GET" })
  .validator(ProductIdSchema)
  .handler(async ({ data }) => {
    return db.select().from(productsTable).where(eq(productsTable.id, data.id)).then(r => r[0] ?? null);
  });

export const createProduct = createServerFn({ method: "POST" })
  .validator(ProductInputSchema)
  .handler(async ({ data }) => {
    return db.insert(productsTable).values(data).returning().then(r => r[0]);
  });

export const updateProduct = createServerFn({ method: "POST" })
  .validator(ProductInputSchema.extend({ id: z.number() }))
  .handler(async ({ data }) => {
    const { id, ...rest } = data;
    return db.update(productsTable).set(rest).where(eq(productsTable.id, id)).returning().then(r => r[0]);
  });

export const deleteProduct = createServerFn({ method: "POST" })
  .validator(ProductIdSchema)
  .handler(async ({ data }) => {
    return db.delete(productsTable).where(eq(productsTable.id, data.id)).returning().then(r => r[0]);
  });
