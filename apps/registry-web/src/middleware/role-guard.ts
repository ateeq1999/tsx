import { redirect } from "@tanstack/react-router"
import { getSession } from "@/server/auth/queries"

export async function requireRole(role: string) {
    const session = await getSession()

    if (!session?.user) {
        throw redirect({ to: "/auth/login" })
    }

    if (session.user.role !== role) {
        throw redirect({ to: "/" })
    }

    return session
}