import { queryOptions, useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { getItems, createItem, updateItem, deleteItem } from "#/server/items"

export const itemsQueryOptions = queryOptions({
  queryKey: ["items"],
  queryFn: () => getItems(),
})

export function useItems() {
  return useQuery(itemsQueryOptions)
}

export function useCreateItem() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: createItem,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["items"] }),
  })
}

export function useUpdateItem() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: updateItem,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["items"] }),
  })
}

export function useDeleteItem() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: deleteItem,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["items"] }),
  })
}
