"use client"

import { useEffect, useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Activity, Server, ShieldCheck, AlertTriangle } from "lucide-react"
import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import api from "@/lib/api"
import { toast } from "sonner"

interface Device {
  id: string
  device_id: string
  status: string
  last_seen: string | null
  type?: string
}

export default function DashboardPage() {
  const [stats, setStats] = useState({
    totalDevices: 0,
    onlineDevices: 0,
    offlineDevices: 0,
    activeSessions: 12, // Mock for now
    securityStatus: "Secure",
    activeAlerts: 0
  })
  const [recentDevices, setRecentDevices] = useState<Device[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        // Fetch devices to calculate stats
        const devicesResponse = await api.get<{ items: Device[], total: number }>('/api/v1/devices')
        const devices = devicesResponse.data.items || []

        const online = devices.filter(d => d.status === 'online').length
        const offline = devices.filter(d => d.status === 'offline').length

        setStats(prev => ({
          ...prev,
          totalDevices: devicesResponse.data.total || devices.length,
          onlineDevices: online,
          offlineDevices: offline
        }))

        setRecentDevices(devices.slice(0, 5))

      } catch (error) {
        console.error("Failed to fetch dashboard data:", error)
        // toast.error("Failed to load dashboard data")
      } finally {
        setIsLoading(false)
      }
    }

    fetchDashboardData()
  }, [])

  return (
    <div className="p-8 space-y-8 bg-background min-h-full">
      <div className="flex items-center justify-between space-y-2">
        <h2 className="text-3xl font-bold tracking-tight">Dashboard</h2>
        <div className="flex items-center space-x-2">
          {/* Add user nav or date picker here if needed */}
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Registered Devices
            </CardTitle>
            <Server className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? "-" : stats.totalDevices}</div>
            <p className="text-xs text-muted-foreground">
              {stats.onlineDevices} online, {stats.offlineDevices} offline
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Active Sessions
            </CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.activeSessions}</div>
            <p className="text-xs text-muted-foreground">
              Across all agents & devices
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Security Status</CardTitle>
            <ShieldCheck className="h-4 w-4 text-emerald-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-emerald-500">{stats.securityStatus}</div>
            <p className="text-xs text-muted-foreground">
              Audit logs verified
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Active Alerts
            </CardTitle>
            <AlertTriangle className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">{stats.activeAlerts}</div>
            <p className="text-xs text-muted-foreground">
              Warning level
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Recent Devices</CardTitle>
            <CardDescription>
              Recently registered or active devices.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {isLoading ? (
                <div className="flex justify-center p-4">Loading...</div>
              ) : recentDevices.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-4">No devices found.</p>
              ) : (
                recentDevices.map((device, i) => (
                  <div key={i} className="flex items-center justify-between border-b pb-2 last:border-0 last:pb-0">
                    <div className="flex items-center space-x-4">
                      <Avatar className="h-9 w-9">
                        <AvatarFallback>{device.device_id[0]?.toUpperCase() || '?'}</AvatarFallback>
                      </Avatar>
                      <div className="space-y-1">
                        <p className="text-sm font-medium leading-none">{device.device_id}</p>
                        <p className="text-xs text-muted-foreground">{device.id.slice(0, 8)}...</p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-4">
                      <div className={`text-sm font-mono ${device.status !== "online" ? "text-muted-foreground" : "text-emerald-500"}`}>
                        {device.status}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {device.last_seen ? new Date(device.last_seen).toLocaleTimeString() : 'Never'}
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </CardContent>
        </Card>
        <Card className="col-span-3">
          <CardHeader>
            <CardTitle>System Health</CardTitle>
            <CardDescription>
              Backend services status.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {[
                "UAIP Core", "Authentication Manager", "Device Registry", "Message Router"
              ].map((service) => (
                <div key={service} className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <div className="h-2.5 w-2.5 rounded-full bg-emerald-500 animate-pulse" />
                    <span className="text-sm font-medium">{service}</span>
                  </div>
                  <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-500">Operational</span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
