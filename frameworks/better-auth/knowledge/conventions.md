# Better Auth — Conventions

- Auth config lives in `lib/auth.ts`, client in `lib/auth-client.ts`
- Always set `BETTER_AUTH_SECRET` in your `.env` file
- Provider credentials follow `<PROVIDER>_CLIENT_ID` / `<PROVIDER>_CLIENT_SECRET` naming
- Use `auth.api.getSession({ headers })` in server functions, `useSession()` in components
