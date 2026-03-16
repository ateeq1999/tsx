import { createFileRoute } from "@tanstack/react-router";
import { productsQueryOptions } from "~/hooks/use-products";
import { ProductsTable } from "~/components/ProductsTable";
import { ProductsForm } from "~/components/ProductsForm";
import { useProductsMutation } from "~/hooks/use-products";
import { useState } from "react";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/products/")({
  loader: ({ context: { queryClient } }) =>
    queryClient.ensureQueryData(productsQueryOptions),

  component: ProductsPage,
});

function ProductsPage() {
  const [showForm, setShowForm] = useState(false);
  const { create } = useProductsMutation();

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Products</h1>
        <Button onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancel" : "New Product"}
        </Button>
      </div>

      {showForm && (
        <div className="rounded-lg border p-6 max-w-md">
          <h2 className="text-lg font-semibold mb-4">Add Product</h2>
          <ProductsForm
            onSubmit={(values) => {
              create.mutate({ data: values }, { onSuccess: () => setShowForm(false) });
            }}
          />
        </div>
      )}

      <ProductsTable />
    </div>
  );
}
