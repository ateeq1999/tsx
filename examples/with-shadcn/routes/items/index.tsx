import { createFileRoute } from "@tanstack/react-router"
import { useItems, useDeleteItem, itemsQueryOptions } from "#/features/items/hooks/use-items"
import { DataTable } from "#/components/ui/data-table"
import type { ColumnDef } from "@tanstack/react-table"
import type { Item } from "#/db/schema/items"
import { Button } from "#/components/ui/button"
import { Badge } from "#/components/ui/badge"
import { Trash2 } from "lucide-react"
import { toast } from "sonner"

export const Route = createFileRoute("/items/")({
  loader: ({ context: { queryClient } }) => queryClient.ensureQueryData(itemsQueryOptions),
  component: ItemsPage,
})

function ItemsPage() {
  const { data: items = [] } = useItems()
  const { mutate: deleteItem } = useDeleteItem()

  const columns: ColumnDef<Item>[] = [
    { accessorKey: "title", header: "Title" },
    {
      accessorKey: "completed",
      header: "Status",
      cell: ({ row }) => (
        <Badge variant={row.original.completed ? "default" : "secondary"}>
          {row.original.completed ? "Done" : "Pending"}
        </Badge>
      ),
    },
    {
      accessorKey: "createdAt",
      header: "Created",
      cell: ({ row }) => new Date(row.original.createdAt).toLocaleDateString(),
    },
    {
      id: "actions",
      cell: ({ row }) => (
        <Button
          variant="ghost"
          size="icon"
          onClick={() => {
            deleteItem({ id: row.original.id })
            toast.success("Item deleted")
          }}
        >
          <Trash2 className="size-4" />
        </Button>
      ),
    },
  ]

  return (
    <div className="p-6">
      <h1 className="mb-6 text-2xl font-bold">Items</h1>
      <DataTable columns={columns} data={items} />
    </div>
  )
}
