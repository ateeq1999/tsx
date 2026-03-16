# Better Auth Overview

Better Auth is a modern authentication library for TypeScript projects. It provides a complete auth solution with session management, multiple providers, and database adapters.

## Key Concepts

- **Client**: The frontend authentication client
- **Server**: The backend auth server configuration
- **Session**: User session management with secure cookies
- **Plugins**: Extend functionality (OAuth, 2FA, email/password)

## With TanStack Start

Better Auth integrates with TanStack Start through:
- Server-side session validation via middleware
- Client-side auth hooks (`useSession`, `signIn`, `signOut`)
- Route protection with `requireAuth`

## Database Schema

Better Auth requires its own tables (sessions, users, accounts). When used with Drizzle ORM, these are automatically generated via the `add:auth-setup` command.
