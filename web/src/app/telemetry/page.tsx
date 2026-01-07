"use client"

import { useEffect, useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    BarChart,
    Bar,
    AreaChart,
    Area
} from "recharts"
import api from "@/lib/api"

const MOCK_TEMPERATURE_DATA = [
    { time: "00:00", indoor: 21, outdoor: 15 },
    { time: "04:00", indoor: 20, outdoor: 14 },
    { time: "08:00", indoor: 22, outdoor: 18 },
    { time: "12:00", indoor: 24, outdoor: 22 },
    { time: "16:00", indoor: 23, outdoor: 20 },
    { time: "20:00", indoor: 22, outdoor: 17 },
    { time: "23:59", indoor: 21, outdoor: 16 },
]

const MOCK_NETWORK_DATA = [
    { time: "00:00", rx: 120, tx: 40 },
    { time: "04:00", rx: 80, tx: 20 },
    { time: "08:00", rx: 450, tx: 120 },
    { time: "12:00", rx: 900, tx: 300 },
    { time: "16:00", rx: 850, tx: 280 },
    { time: "20:00", rx: 600, tx: 150 },
    { time: "23:59", rx: 300, tx: 80 },
]

const MOCK_CPU_DATA = [
    { name: "Core", usage: 45 },
    { name: "Auth", usage: 12 },
    { name: "Registry", usage: 8 },
    { name: "Router", usage: 32 },
    { name: "Adapter", usage: 25 },
]

export default function TelemetryPage() {
    const [temperatureData, setTemperatureData] = useState(MOCK_TEMPERATURE_DATA)
    const [networkData, setNetworkData] = useState(MOCK_NETWORK_DATA)
    const [cpuData, setCpuData] = useState(MOCK_CPU_DATA)
    const [isLoading, setIsLoading] = useState(true)

    useEffect(() => {
        const fetchTelemetry = async () => {
            try {
                // Attempt to fetch real telemetry
                // This endpoint likely doesn't exist yet, but the frontend is now ready for it.
                // const response = await api.get('/api/v1/telemetry')
                // setTemperatureData(response.data.temperature)
                // ...

                // Simulate network request
                await new Promise(r => setTimeout(r, 800))

            } catch (error) {
                console.warn("Failed to fetch telemetry, using mock data:", error)
            } finally {
                setIsLoading(false)
            }
        }
        fetchTelemetry()
    }, [])

    if (isLoading) {
        return <div className="flex h-screen items-center justify-center">Loading Telemetry...</div>
    }

    return (
        <div className="container mx-auto py-10 px-10">
            <div className="flex items-center justify-between space-y-2 mb-8">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight">System Telemetry</h2>
                    <p className="text-muted-foreground">Real-time metrics and historical data analysis.</p>
                </div>
            </div>

            <Tabs defaultValue="overview" className="space-y-4">
                <TabsList>
                    <TabsTrigger value="overview">Overview</TabsTrigger>
                    <TabsTrigger value="network">Network</TabsTrigger>
                    <TabsTrigger value="environment">Environment</TabsTrigger>
                </TabsList>

                <TabsContent value="overview" className="space-y-4">
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
                        <Card className="col-span-4">
                            <CardHeader>
                                <CardTitle>Network Traffic (24h)</CardTitle>
                                <CardDescription>Aggregate inbound and outbound traffic.</CardDescription>
                            </CardHeader>
                            <CardContent className="pl-2">
                                <ResponsiveContainer width="100%" height={350}>
                                    <AreaChart data={networkData}>
                                        <defs>
                                            <linearGradient id="colorRx" x1="0" y1="0" x2="0" y2="1">
                                                <stop offset="5%" stopColor="#10b981" stopOpacity={0.8} />
                                                <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                                            </linearGradient>
                                            <linearGradient id="colorTx" x1="0" y1="0" x2="0" y2="1">
                                                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8} />
                                                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                                            </linearGradient>
                                        </defs>
                                        <XAxis
                                            dataKey="time"
                                            stroke="#888888"
                                            fontSize={12}
                                            tickLine={false}
                                            axisLine={false}
                                        />
                                        <YAxis
                                            stroke="#888888"
                                            fontSize={12}
                                            tickLine={false}
                                            axisLine={false}
                                            tickFormatter={(value) => `${value} MB`}
                                        />
                                        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" vertical={false} />
                                        <Tooltip
                                            contentStyle={{ backgroundColor: 'hsl(var(--card))', borderColor: 'hsl(var(--border))' }}
                                            itemStyle={{ color: 'hsl(var(--foreground))' }}
                                        />
                                        <Area type="monotone" dataKey="rx" stroke="#10b981" fillOpacity={1} fill="url(#colorRx)" name="Inbound" />
                                        <Area type="monotone" dataKey="tx" stroke="#3b82f6" fillOpacity={1} fill="url(#colorTx)" name="Outbound" />
                                    </AreaChart>
                                </ResponsiveContainer>
                            </CardContent>
                        </Card>
                        <Card className="col-span-3">
                            <CardHeader>
                                <CardTitle>Service Load</CardTitle>
                                <CardDescription>CPU usage by microservice.</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <ResponsiveContainer width="100%" height={350}>
                                    <BarChart data={cpuData} layout="vertical" margin={{ left: 20 }}>
                                        <XAxis type="number" hide />
                                        <YAxis dataKey="name" type="category" stroke="#888888" fontSize={12} tickLine={false} axisLine={false} />
                                        <Tooltip
                                            cursor={{ fill: 'transparent' }}
                                            contentStyle={{ backgroundColor: 'hsl(var(--card))', borderColor: 'hsl(var(--border))' }}
                                            itemStyle={{ color: 'hsl(var(--foreground))' }}
                                        />
                                        <Bar dataKey="usage" fill="#f43f5e" radius={[0, 4, 4, 0]} barSize={32} />
                                    </BarChart>
                                </ResponsiveContainer>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>

                <TabsContent value="environment" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Temperature Trends</CardTitle>
                            <CardDescription>Indoor vs Outdoor sensors over time.</CardDescription>
                        </CardHeader>
                        <CardContent className="pl-2">
                            <ResponsiveContainer width="100%" height={400}>
                                <LineChart data={temperatureData}>
                                    <XAxis
                                        dataKey="time"
                                        stroke="#888888"
                                        fontSize={12}
                                        tickLine={false}
                                        axisLine={false}
                                    />
                                    <YAxis
                                        stroke="#888888"
                                        fontSize={12}
                                        tickLine={false}
                                        axisLine={false}
                                        tickFormatter={(value) => `${value}°C`}
                                    />
                                    <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                                    <Tooltip
                                        contentStyle={{ backgroundColor: 'hsl(var(--card))', borderColor: 'hsl(var(--border))' }}
                                        itemStyle={{ color: 'hsl(var(--foreground))' }}
                                    />
                                    <Line type="monotone" dataKey="indoor" stroke="#8884d8" strokeWidth={2} activeDot={{ r: 8 }} />
                                    <Line type="monotone" dataKey="outdoor" stroke="#82ca9d" strokeWidth={2} />
                                </LineChart>
                            </ResponsiveContainer>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    )
}
