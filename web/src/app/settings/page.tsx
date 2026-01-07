"use client"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { Separator } from "@/components/ui/separator"
import { Save, User } from "lucide-react"
import { useAuthStore } from "@/lib/store"
import { UsersTab } from "@/components/settings/users-tab"

export default function SettingsPage() {
    const { user } = useAuthStore()

    return (
        <div className="container mx-auto py-10 px-10">
            <div className="flex items-center justify-between space-y-2 mb-8">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight">Settings</h2>
                    <p className="text-muted-foreground">Manage your account and system preferences.</p>
                </div>
                <Button>
                    <Save className="mr-2 h-4 w-4" /> Save Changes
                </Button>
            </div>

            <Tabs defaultValue="profile" className="space-y-4">
                <TabsList>
                    <TabsTrigger value="profile">Profile</TabsTrigger>
                    <TabsTrigger value="users">Users</TabsTrigger>
                    <TabsTrigger value="general">General</TabsTrigger>
                    <TabsTrigger value="adapters">Adapters</TabsTrigger>
                    <TabsTrigger value="notifications">Notifications</TabsTrigger>
                </TabsList>

                <TabsContent value="profile">
                    <Card>
                        <CardHeader>
                            <CardTitle>User Profile</CardTitle>
                            <CardDescription>
                                Manage your personal account settings.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex items-center gap-4 mb-4">
                                <div className="h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center text-primary text-2xl font-bold">
                                    {user?.email?.[0].toUpperCase() || "U"}
                                </div>
                                <div>
                                    <h3 className="text-lg font-medium">{user?.email || "User"}</h3>
                                    <p className="text-sm text-muted-foreground capitalize">{user?.role || "Admin"}</p>
                                </div>
                            </div>
                            <Separator />
                            <div className="space-y-2">
                                <Label htmlFor="email">Email Address</Label>
                                <Input id="email" value={user?.email || ""} disabled />
                                <p className="text-[0.8rem] text-muted-foreground">
                                    Your email address is managed by the administrator.
                                </p>
                            </div>
                            <div className="space-y-2">
                                <Label htmlFor="role">Role</Label>
                                <Input id="role" value={user?.role || "admin"} disabled />
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="general">
                    <div className="grid gap-6">
                        <Card>
                            <CardHeader>
                                <CardTitle>Hub Configuration</CardTitle>
                                <CardDescription>Basic system identity and location.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <Label htmlFor="hubName">Hub Name</Label>
                                    <Input id="hubName" defaultValue="UAIP-Main-Hub" />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="location">Physical Location</Label>
                                    <Input id="location" defaultValue="Server Room A, Building 1" />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="timezone">Timezone</Label>
                                    <Select defaultValue="utc">
                                        <SelectTrigger id="timezone">
                                            <SelectValue placeholder="Select timezone" />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="utc">UTC (Coordinated Universal Time)</SelectItem>
                                            <SelectItem value="est">EST (Eastern Standard Time)</SelectItem>
                                            <SelectItem value="pst">PST (Pacific Standard Time)</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardHeader>
                                <CardTitle>Network</CardTitle>
                                <CardDescription>Connection settings.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="flex items-center justify-between space-x-2">
                                    <Label htmlFor="ipv6" className="flex flex-col space-y-1">
                                        <span>IPv6 Support</span>
                                        <span className="font-normal leading-snug text-muted-foreground">
                                            Enable IPv6 network stack.
                                        </span>
                                    </Label>
                                    <Switch id="ipv6" />
                                </div>
                                <Separator />
                                <div className="flex items-center justify-between space-x-2">
                                    <Label htmlFor="discovery" className="flex flex-col space-y-1">
                                        <span>mDNS Discovery</span>
                                        <span className="font-normal leading-snug text-muted-foreground">
                                            Allow devices to discover hub automatically.
                                        </span>
                                    </Label>
                                    <Switch id="discovery" defaultChecked />
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>

                <TabsContent value="users">
                    <Card>
                        <CardHeader>
                            <CardTitle>User Management</CardTitle>
                            <CardDescription>Manage system access, roles, and permissions.</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <UsersTab />
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="adapters">
                    <Card>
                        <CardHeader>
                            <CardTitle>Protocol Adapters</CardTitle>
                            <CardDescription>
                                Configure enabled communication protocols.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            {["MQTT", "Zigbee", "Modbus TCP", "OPC-UA", "WebRTC", "RTSP"].map((protocol) => (
                                <div key={protocol} className="flex items-center justify-between space-x-2">
                                    <Label className="flex flex-col space-y-1">
                                        <span>{protocol}</span>
                                        <span className="font-normal leading-snug text-muted-foreground">
                                            Enable support for {protocol} devices.
                                        </span>
                                    </Label>
                                    <Switch defaultChecked={["MQTT", "WebRTC"].includes(protocol)} />
                                </div>
                            ))}
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="notifications">
                    <Card>
                        <CardHeader>
                            <CardTitle>Alert Rules</CardTitle>
                            <CardDescription>Configure when to send notifications.</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="space-y-2">
                                <Label>Notification Email</Label>
                                <Input type="email" placeholder="admin@example.com" />
                            </div>
                            <div className="space-y-2">
                                <Label>Webhook URL</Label>
                                <Input placeholder="https://..." />
                            </div>
                            <Separator />
                            <div className="space-y-4">
                                {[
                                    "Device Offline > 5 mins",
                                    "Security Certificate Expiration",
                                    "AI Agent Error",
                                    "High CPU Load (>90%)"
                                ].map((rule) => (
                                    <div key={rule} className="flex items-start space-x-3">
                                        <Switch id={rule} defaultChecked />
                                        <Label htmlFor={rule}>{rule}</Label>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    )
}
