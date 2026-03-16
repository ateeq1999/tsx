import { queryOptions, useQuery } from "@tanstack/react-query"
import { getSubscription, getUsage } from "#/server/billing"

export const subscriptionQueryOptions = queryOptions({
  queryKey: ["billing", "subscription"],
  queryFn: () => getSubscription(),
})

export const usageQueryOptions = queryOptions({
  queryKey: ["billing", "usage"],
  queryFn: () => getUsage(),
})

export function useSubscription() {
  return useQuery(subscriptionQueryOptions)
}

export function useUsage() {
  return useQuery(usageQueryOptions)
}
