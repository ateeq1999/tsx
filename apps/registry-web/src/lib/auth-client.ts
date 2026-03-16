import { createAuthClient } from "better-auth/react";

export const authClient = createAuthClient({
	baseURL: process.env.VITE_SERVER_URL || "http://localhost:3000",
});
