"use client"

import Link from "next/link"
import { usePathname, useRouter } from "next/navigation"
import { cn } from "@/lib/utils"
import {
    LayoutDashboard,
    Server,
    Shield,
    Activity,
    Settings,
    Menu,
    Cpu,
    LogOut,
    User,
    LogIn
} from "lucide-react"

import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { useAuthStore } from "@/lib/store"

const routes = [
    {
        label: "Dashboard",
        icon: LayoutDashboard,
        href: "/",
        color: "text-sky-500",
    },
    {
        label: "Device Registry",
        icon: Server,
        href: "/devices",
        color: "text-violet-500",
    },
    {
        label: "Intelligence",
        icon: Cpu,
        href: "/intelligence",
        color: "text-emerald-500",
    },
    {
        label: "Security",
        icon: Shield,
        href: "/security",
        color: "text-pink-700",
    },
    {
        label: "Telemetry",
        icon: Activity,
        href: "/telemetry",
        color: "text-orange-700",
    },
    {
        label: "Settings",
        icon: Settings,
        href: "/settings",
    },
]

interface SidebarProps extends React.HTMLAttributes<HTMLDivElement> { }

export function AppSidebar({ className }: SidebarProps) {
    const pathname = usePathname()
    const router = useRouter()
    const { user, logout, isAuthenticated } = useAuthStore()

    const handleLogout = () => {
        logout()
        router.push("/login")
    }

    return (
        <div className={cn("pb-12 h-full border-r bg-card flex flex-col justify-between", className)}>
            <div className="space-y-4 py-4">
                <div className="px-3 py-2">
                    <div className="flex items-center px-4 mb-6">
                        <Cpu className="h-6 w-6 mr-2" />
                        <h2 className="text-xl font-bold tracking-tight">
                            UAIP Hub
                        </h2>
                    </div>
                    <div className="space-y-1">
                        {routes.map((route) => (
                            <Button
                                key={route.href}
                                variant={pathname === route.href ? "secondary" : "ghost"}
                                className={cn("w-full justify-start", pathname === route.href && "bg-muted")}
                                asChild
                            >
                                <Link href={route.href}>
                                    <route.icon className={cn("mr-2 h-4 w-4", route.color)} />
                                    {route.label}
                                </Link>
                            </Button>
                        ))}
                    </div>
                </div>
            </div>

            <div className="px-3 py-2">
                {isAuthenticated ? (
                    <>
                        <div className="flex items-center gap-3 px-4 py-2 mb-2 rounded-md bg-muted/50">
                            <div className="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center text-primary text-xs font-bold">
                                {user?.email?.[0].toUpperCase() || "U"}
                            </div>
                            <div className="flex flex-col overflow-hidden">
                                <span className="text-sm font-medium truncate max-w-[140px]">{user?.email || "Guest"}</span>
                                <span className="text-xs text-muted-foreground capitalize">{user?.role || "Viewer"}</span>
                            </div>
                        </div>
                        <Button
                            variant="ghost"
                            className="w-full justify-start text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                            onClick={handleLogout}
                        >
                            <LogOut className="mr-2 h-4 w-4" />
                            Logout
                        </Button>
                    </>
                ) : (
                    <Button
                        variant="default"
                        className="w-full justify-start"
                        asChild
                    >
                        <Link href="/auth/login">
                            <LogIn className="mr-2 h-4 w-4" />
                            Log In
                        </Link>
                    </Button>
                )}
            </div>
        </div>
    )
}

export function MobileSidebar() {
    return (
        <Sheet>
            <SheetTrigger asChild>
                <Button variant="ghost" size="icon" className="md:hidden">
                    <Menu className="h-5 w-5" />
                </Button>
            </SheetTrigger>
            <SheetContent side="left" className="p-0 w-72">
                <AppSidebar />
            </SheetContent>
        </Sheet>
    )
}
