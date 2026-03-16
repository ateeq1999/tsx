import { createQueryOptions } from "@tanstack/react-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { listProducts, createProduct, updateProduct, deleteProduct } from "~/server-functions/products";

export const productsQueryKey = ["products"] as const;

export const productsQueryOptions = createQueryOptions({
  queryKey: productsQueryKey,
  queryFn: () => listProducts(),
});

export function useProductsMutation() {
  const queryClient = useQueryClient();

  const create = useMutation({
    mutationFn: createProduct,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: productsQueryKey }),
  });

  const update = useMutation({
    mutationFn: updateProduct,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: productsQueryKey }),
  });

  const remove = useMutation({
    mutationFn: deleteProduct,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: productsQueryKey }),
  });

  return { create, update, remove };
}
