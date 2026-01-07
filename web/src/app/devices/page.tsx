"use client"

import { useEffect, useState } from "react"
import { DeviceInfo } from "@/types/api"
import { columns } from "./columns"
import { DataTable } from "@/components/ui/data-table"
import api from "@/lib/api"
import { Loader2 } from "lucide-react"
import { toast } from "sonner"

export default function DevicesPage() {
    const [data, setData] = useState<DeviceInfo[]>([])
    const [isLoading, setIsLoading] = useState(true)

    useEffect(() => {
        const fetchDevices = async () => {
            try {
                const response = await api.get('/api/v1/devices')
                // Backend returns { devices: [...], total: ... }
                setData(response.data.devices || [])
            } catch (error) {
                console.error("Failed to fetch devices:", error)
                toast.error("Failed to load devices", {
                    description: "Could not connect to the device registry."
                })
                // Fallback to empty or mock data if needed, but for now empty
            } finally {
                setIsLoading(false)
            }
        }

        fetchDevices()
    }, [])

    return (
        <div className="container mx-auto py-10 px-10">
            <div className="flex items-center justify-between space-y-2 mb-8">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight">Devices</h2>
                    <p className="text-muted-foreground">Manage and monitor connected hardware.</p>
                </div>
            </div>

            {isLoading ? (
                <div className="flex h-[400px] items-center justify-center">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
            ) : (
                <DataTable columns={columns} data={data} />
            )}
        </div>
    )
}
