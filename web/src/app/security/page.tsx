"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ShieldCheck, Lock, FileKey, History, AlertOctagon } from "lucide-react"

export default function SecurityPage() {
    return (
        <div className="container mx-auto py-10 px-10">
            <div className="flex items-center justify-between space-y-2 mb-8">
                <div>
                    <h2 className="text-3xl font-bold tracking-tight">Security Center</h2>
                    <p className="text-muted-foreground">Manage certificates, access control, and audit logs.</p>
                </div>
                <div className="flex items-center space-x-2">
                    <Badge variant="outline" className="text-emerald-500 bg-emerald-500/10 border-emerald-500/20 px-3 py-1">
                        <ShieldCheck className="w-4 h-4 mr-2" /> System Secure
                    </Badge>
                </div>
            </div>

            <Tabs defaultValue="certificates" className="space-y-4">
                <TabsList>
                    <TabsTrigger value="certificates">Certificates</TabsTrigger>
                    <TabsTrigger value="access">Access Control</TabsTrigger>
                    <TabsTrigger value="audit">Audit Logs</TabsTrigger>
                </TabsList>
                <TabsContent value="certificates" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Active Certificates</CardTitle>
                            <CardDescription>
                                Manage device and client certificates.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-4">
                                {[
                                    { id: "cert_8x92", subject: "SmartCam-3000", issuer: "UAIP CA", expires: "2025-12-31", status: "valid" },
                                    { id: "cert_k2m1", subject: "Agent-Vision-1", issuer: "UAIP CA", expires: "2024-05-20", status: "expiring_soon" },
                                    { id: "cert_99aa", subject: "Old-Sensor-X", issuer: "Internal CA", expires: "2023-01-01", status: "revoked" },
                                ].map((cert) => (
                                    <div key={cert.id} className="flex items-center justify-between border-b last:border-0 pb-4 last:pb-0">
                                        <div className="flex items-center space-x-4">
                                            <div className="p-2 bg-muted rounded-full">
                                                <FileKey className="w-5 h-5" />
                                            </div>
                                            <div>
                                                <p className="font-medium">{cert.subject}</p>
                                                <p className="text-xs text-muted-foreground">ID: {cert.id} • Issued by {cert.issuer}</p>
                                            </div>
                                        </div>
                                        <div className="flex items-center space-x-4">
                                            <div className="text-sm text-right">
                                                <p className="text-muted-foreground text-xs">Expires</p>
                                                <p>{cert.expires}</p>
                                            </div>
                                            <Badge variant={cert.status === 'valid' ? 'default' : cert.status === 'revoked' ? 'destructive' : 'secondary'}>
                                                {cert.status}
                                            </Badge>
                                            <Button variant="ghost" size="sm">Manage</Button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>
                <TabsContent value="access" className="space-y-4">
                    <div className="grid gap-4 md:grid-cols-2">
                        <Card>
                            <CardHeader>
                                <CardTitle>Role Based Access</CardTitle>
                                <CardDescription>Defined roles and their permissions.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center space-x-2">
                                        <Lock className="w-4 h-4 text-muted-foreground" />
                                        <span className="font-medium">Administrator</span>
                                    </div>
                                    <Badge variant="outline">Full Access</Badge>
                                </div>
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center space-x-2">
                                        <Lock className="w-4 h-4 text-muted-foreground" />
                                        <span className="font-medium">Operator</span>
                                    </div>
                                    <Badge variant="outline">Read/Write</Badge>
                                </div>
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center space-x-2">
                                        <Lock className="w-4 h-4 text-muted-foreground" />
                                        <span className="font-medium">Viewer</span>
                                    </div>
                                    <Badge variant="outline">Read Only</Badge>
                                </div>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardHeader>
                                <CardTitle>API Access</CardTitle>
                                <CardDescription>Active API keys and tokens.</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-center py-8 text-muted-foreground">
                                    No external API keys generated.
                                </div>
                                <Button className="w-full" variant="outline">Generate New Key</Button>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>
                <TabsContent value="audit" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>Recent Activity</CardTitle>
                            <CardDescription>
                                Security events and access logs.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-6">
                                {[
                                    { event: "Login Success", user: "diallo@uaip.admin", ip: "192.168.1.50", time: "Just now", icon: ShieldCheck, color: "text-emerald-500" },
                                    { event: "Config Change", user: "system", ip: "localhost", time: "1 hour ago", icon: Settings2, color: "text-blue-500" },
                                    { event: "Failed Login", user: "unknown", ip: "10.0.0.99", time: "3 hours ago", icon: AlertOctagon, color: "text-red-500" },
                                    { event: "Device Registered", user: "SmartCam-3000", ip: "192.168.1.102", time: "5 hours ago", icon: History, color: "text-muted-foreground" },
                                ].map((log, i) => (
                                    <div key={i} className="flex items-start justify-between">
                                        <div className="flex items-start space-x-4">
                                            <log.icon className={`w-5 h-5 mt-0.5 ${log.color}`} />
                                            <div>
                                                <p className="text-sm font-medium">{log.event}</p>
                                                <p className="text-xs text-muted-foreground">{log.user} • {log.ip}</p>
                                            </div>
                                        </div>
                                        <span className="text-xs text-muted-foreground">{log.time}</span>
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

import { Settings2 } from "lucide-react"

