import { createColumnHelper } from "@tanstack/react-table";
import { useQuery } from "@tanstack/react-query";
import { DataTable } from "@/components/ui/data-table";
import { Button } from "@/components/ui/button";
import { productsQueryOptions } from "~/hooks/use-products";
import type { Product } from "~/db/schema/products";

const columnHelper = createColumnHelper<Product>();

const columns = [
  columnHelper.accessor("title", {
    header: "Title",
  }),
  columnHelper.accessor("price", {
    header: "Price",
    cell: ({ getValue }) => `$${getValue()}`,
  }),
  columnHelper.accessor("in_stock", {
    header: "In Stock",
    cell: ({ getValue }) => (getValue() ? "Yes" : "No"),
  }),
  columnHelper.display({
    id: "actions",
    header: "Actions",
    cell: ({ row }) => (
      <div className="flex gap-2">
        <Button variant="outline" size="sm">Edit</Button>
        <Button variant="destructive" size="sm">Delete</Button>
      </div>
    ),
  }),
];

export function ProductsTable() {
  const { data, isLoading } = useQuery(productsQueryOptions);

  return (
    <DataTable
      columns={columns}
      data={data ?? []}
      isLoading={isLoading}
    />
  );
}
