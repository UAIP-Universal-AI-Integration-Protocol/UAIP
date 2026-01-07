"use client"

import { ColumnDef } from "@tanstack/react-table"
import { DeviceInfo } from "@/types/api"
import { Badge } from "@/components/ui/badge"
import { Checkbox } from "@/components/ui/checkbox"
import { Button } from "@/components/ui/button"
import { ArrowUpDown, MoreHorizontal } from "lucide-react"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

export const columns: ColumnDef<DeviceInfo>[] = [
    {
        id: "select",
        header: ({ table }) => (
            <Checkbox
                checked={
                    table.getIsAllPageRowsSelected() ||
                    (table.getIsSomePageRowsSelected() && "indeterminate")
                }
                onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
                aria-label="Select all"
            />
        ),
        cell: ({ row }) => (
            <Checkbox
                checked={row.getIsSelected()}
                onCheckedChange={(value) => row.toggleSelected(!!value)}
                aria-label="Select row"
            />
        ),
        enableSorting: false,
        enableHiding: false,
    },
    {
        accessorKey: "name",
        header: ({ column }) => {
            return (
                <Button
                    variant="ghost"
                    onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
                >
                    Name
                    <ArrowUpDown className="ml-2 h-4 w-4" />
                </Button>
            )
        },
        cell: ({ row }) => <div className="font-medium ml-4">{row.getValue("name")}</div>,
    },
    {
        accessorKey: "device_type",
        header: "Type",
        cell: ({ row }) => {
            const type = row.getValue("device_type") as string;
            return (
                <div className="capitalize">{type}</div>
            )
        }
    },
    {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => {
            const status = row.getValue("status") as string
            return (
                <Badge
                    variant={
                        status === "online"
                            ? "default"
                            : status === "maintenance"
                                ? "secondary"
                                : status === "error"
                                    ? "destructive"
                                    : "outline"
                    }
                    className={
                        status === "online" ? "bg-emerald-500 hover:bg-emerald-600" : ""
                    }
                >
                    {status}
                </Badge>
            )
        },
    },
    {
        accessorKey: "last_seen",
        header: "Last Seen",
        cell: ({ row }) => {
            const date = row.getValue("last_seen")
            if (!date) return <div className="text-muted-foreground">-</div>
            return <div>{new Date(date as string).toLocaleString()}</div>
        },
    },
    {
        id: "actions",
        cell: ({ row }) => {
            const device = row.original

            return (
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" className="h-8 w-8 p-0">
                            <span className="sr-only">Open menu</span>
                            <MoreHorizontal className="h-4 w-4" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        <DropdownMenuLabel>Actions</DropdownMenuLabel>
                        <DropdownMenuItem
                            onClick={() => navigator.clipboard.writeText(device.device_id)}
                        >
                            Copy Device ID
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>View Details</DropdownMenuItem>
                        <DropdownMenuItem>View Telemetry</DropdownMenuItem>
                        <DropdownMenuItem className="text-red-500">Decommission</DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            )
        },
    },
]
