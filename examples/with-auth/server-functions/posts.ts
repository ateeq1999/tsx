import { createServerFn } from "@tanstack/start/server";
import { db } from "~/db";
import { postsTable } from "~/db/schema/posts";
import { eq } from "drizzle-orm";
import { getSession } from "~/lib/auth";
import { z } from "zod";

const PostInputSchema = z.object({
  title: z.string().min(1),
  content: z.string().optional(),
});

const PostIdSchema = z.object({ id: z.number() });

export const listPosts = createServerFn({ method: "GET" })
  .handler(async () => {
    const session = await getSession();
    if (!session) throw new Error("Unauthorized");
    return db.select().from(postsTable);
  });

export const createPost = createServerFn({ method: "POST" })
  .validator(PostInputSchema)
  .handler(async ({ data }) => {
    const session = await getSession();
    if (!session) throw new Error("Unauthorized");
    return db
      .insert(postsTable)
      .values({ ...data, user_id: session.user.id as unknown as number })
      .returning()
      .then((r) => r[0]);
  });

export const deletePost = createServerFn({ method: "POST" })
  .validator(PostIdSchema)
  .handler(async ({ data }) => {
    const session = await getSession();
    if (!session) throw new Error("Unauthorized");
    return db.delete(postsTable).where(eq(postsTable.id, data.id)).returning().then((r) => r[0]);
  });
