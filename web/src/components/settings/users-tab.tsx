"use client"

import { useEffect, useState } from "react"
import {
    ColumnDef,
    flexRender,
    getCoreRowModel,
    useReactTable,
} from "@tanstack/react-table"
import { MoreHorizontal, Shield, User as UserIcon, Bot, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"
import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import api from "@/lib/api"
import { toast } from "sonner"
import { AddUserDialog } from "./add-user-dialog"
import { EditUserDialog } from "./edit-user-dialog"

interface User {
    id: string
    name: string
    email: string
    role: string
    active: boolean
    last_login: string | null
    created_at: string
}

export function UsersTab() {
    const [data, setData] = useState<User[]>([])
    const [isLoading, setIsLoading] = useState(true)
    const [editingUser, setEditingUser] = useState<User | null>(null)

    const fetchUsers = async () => {
        try {
            setIsLoading(true)
            const response = await api.get<{ users: User[], total: number }>('/api/v1/users')
            setData(response.data.users || [])
        } catch (error) {
            console.error("Failed to fetch users", error)
            toast.error("Failed to load users")
        } finally {
            setIsLoading(false)
        }
    }

    useEffect(() => {
        fetchUsers()
    }, [])

    const handleDelete = async (id: string) => {
        if (!confirm("Are you sure you want to delete this user?")) return
        try {
            await api.delete(`/api/v1/users/${id}`)
            toast.success("User deleted")
            fetchUsers()
        } catch (error) {
            toast.error("Failed to delete user")
        }
    }

    const columns: ColumnDef<User>[] = [
        {
            accessorKey: "name",
            header: "User",
            cell: ({ row }) => (
                <div className="flex items-center gap-3">
                    <Avatar className="h-9 w-9">
                        <AvatarFallback className="uppercase bg-primary/10 text-primary">
                            {row.original.email[0]}
                        </AvatarFallback>
                    </Avatar>
                    <div className="flex flex-col">
                        <span className="font-medium">{row.getValue("name")}</span>
                        <span className="text-xs text-muted-foreground">{row.original.email}</span>
                    </div>
                </div>
            ),
        },
        {
            accessorKey: "role",
            header: "Role",
            cell: ({ row }) => {
                const role = row.getValue("role") as string
                let icon = UserIcon
                let color = "bg-slate-500"

                switch (role) {
                    case "admin": icon = Shield; color = "bg-red-500"; break;
                    case "operator": icon = UserIcon; color = "bg-blue-500"; break;
                    case "ai_agent": icon = Bot; color = "bg-purple-500"; break;
                    default: icon = UserIcon; color = "bg-gray-500"; break;
                }

                const Icon = icon;

                return (
                    <div className="flex items-center gap-2">
                        <Badge variant="outline" className="pl-1 pr-2 gap-1 capitalize">
                            <div className={`p-1 rounded-full ${color} text-white`}>
                                <Icon className="h-3 w-3" />
                            </div>
                            {role.replace("_", " ")}
                        </Badge>
                    </div>
                )
            },
        },
        {
            accessorKey: "last_login",
            header: "Last Login",
            cell: ({ row }) => {
                const lastLogin = row.getValue("last_login") as string | null
                if (!lastLogin) return <span className="text-muted-foreground text-xs">Never</span>
                return <span className="text-xs text-muted-foreground">{new Date(lastLogin).toLocaleString()}</span>
            },
        },
        {
            accessorKey: "active",
            header: "Status",
            cell: ({ row }) => {
                const active = row.getValue("active") as boolean
                return (
                    <Badge variant={active ? 'default' : 'secondary'} className={active ? 'bg-emerald-500 hover:bg-emerald-600' : ''}>
                        {active ? 'Active' : 'Inactive'}
                    </Badge>
                )
            },
        },
        {
            id: "actions",
            enableHiding: false,
            cell: ({ row }) => {
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
                            <DropdownMenuItem onClick={() => navigator.clipboard.writeText(row.original.id)}>
                                Copy ID
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => setEditingUser(row.original)}>
                                Edit User
                            </DropdownMenuItem>
                            <DropdownMenuItem
                                className="text-red-600 focus:text-red-600"
                                onClick={() => handleDelete(row.original.id)}
                            >
                                <Trash2 className="mr-2 h-4 w-4" />
                                Delete User
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                )
            },
        },
    ]

    const table = useReactTable({
        data,
        columns,
        getCoreRowModel: getCoreRowModel(),
    })

    return (
        <div className="space-y-4">
            <div className="flex justify-between items-center">
                <div>
                    <h3 className="text-lg font-medium">Users</h3>
                    <p className="text-sm text-muted-foreground">Manage authorized users.</p>
                </div>
                <AddUserDialog onSuccess={fetchUsers} />
            </div>
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => {
                                    return (
                                        <TableHead key={header.id}>
                                            {header.isPlaceholder
                                                ? null
                                                : flexRender(
                                                    header.column.columnDef.header,
                                                    header.getContext()
                                                )}
                                        </TableHead>
                                    )
                                })}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows?.length ? (
                            table.getRowModel().rows.map((row) => (
                                <TableRow
                                    key={row.id}
                                    data-state={row.getIsSelected() && "selected"}
                                >
                                    {row.getVisibleCells().map((cell) => (
                                        <TableCell key={cell.id}>
                                            {flexRender(
                                                cell.column.columnDef.cell,
                                                cell.getContext()
                                            )}
                                        </TableCell>
                                    ))}
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell
                                    colSpan={columns.length}
                                    className="h-24 text-center"
                                >
                                    {isLoading ? "Loading..." : "No users found."}
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            <EditUserDialog
                open={!!editingUser}
                onOpenChange={(open) => !open && setEditingUser(null)}
                user={editingUser}
                onSuccess={fetchUsers}
            />
        </div>
    )
}
