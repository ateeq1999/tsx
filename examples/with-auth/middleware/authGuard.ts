import { createMiddleware } from "@tanstack/start";
import { auth } from "~/lib/auth";
import { getWebRequest } from "@tanstack/start/server";

export const dashboardGuard = createMiddleware().server(async ({ next }) => {
  const request = getWebRequest();
  const session = await auth.api.getSession({ headers: request.headers });

  if (!session) {
    throw new Response(null, {
      status: 302,
      headers: { Location: "/login" },
    });
  }

  return next({ context: { session } });
});
