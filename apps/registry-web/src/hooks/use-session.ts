import { useQuery } from "@tanstack/react-query"
import { getSession } from "@/server/auth/queries"

export function useSession() {
    return useQuery({
        queryKey: ["session"],
        queryFn: () => getSession()
    })
}