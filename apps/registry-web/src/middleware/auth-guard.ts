import { redirect } from "@tanstack/react-router"
import { getSession } from "@/server/auth/queries"

export async function requireAuth() {
    const session = await getSession()

    if (!session?.user) {
        throw redirect({
            to: "/auth/login"
        })
    }

    return session
}