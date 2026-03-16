import { queryOptions, useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { getOrganizations, createOrganization, deleteOrganization } from "#/server/organizations"

export const organizationsQueryOptions = queryOptions({
  queryKey: ["organizations"],
  queryFn: () => getOrganizations(),
})

export function useOrganizations() {
  return useQuery(organizationsQueryOptions)
}

export function useCreateOrganization() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: createOrganization,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["organizations"] }),
  })
}

export function useDeleteOrganization() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: deleteOrganization,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["organizations"] }),
  })
}
