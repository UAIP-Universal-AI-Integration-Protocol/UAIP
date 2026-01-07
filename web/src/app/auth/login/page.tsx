"use client"

import { useState } from "react"
import Link from "next/link"
import { useRouter } from "next/navigation"
import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form"
import * as z from "zod"
import { Loader2, ShieldCheck, Cpu } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import api from "@/lib/api"
import { useAuthStore } from "@/lib/store"
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from "@/components/ui/form"
import { toast } from "sonner" // Changed from use-toast

const formSchema = z.object({
    email: z.string().email({
        message: "Please enter a valid email address.",
    }),
    password: z.string().min(8, {
        message: "Password must be at least 8 characters.",
    }),
})

export default function LoginPage() {
    const router = useRouter()
    const [isLoading, setIsLoading] = useState(false)
    const login = useAuthStore((state) => state.login)

    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            email: "",
            password: "",
        },
    })

    async function onSubmit(values: z.infer<typeof formSchema>) {
        setIsLoading(true)

        try {
            const response = await api.post('/api/v1/auth/login', {
                grant_type: 'client_credentials',
                client_id: values.email,
                client_secret: values.password
            })

            const { access_token, require_password_change } = response.data

            login(access_token, {
                id: 'current-user',
                email: values.email,
                role: 'admin' // TODO: Get actual role from response
            })

            if (require_password_change) {
                toast.warning("Password Change Required", {
                    description: "For security, you must change your password before proceeding.",
                })
                router.push("/auth/change-password")
                return
            }

            toast.success("Authentication successful", {
                description: "Redirecting to dashboard...",
            })

            router.push("/")
        } catch (error: any) {
            console.error("Login Error:", error)
            toast.error("Authentication failed", {
                description: error.response?.data?.error || "Please check your login details."
            })
        } finally {
            setIsLoading(false)
        }
    }

    return (
        <div className="w-full lg:grid lg:min-h-screen lg:grid-cols-2 xl:min-h-screen">
            <div className="flex items-center justify-center py-12">
                <div className="mx-auto grid w-[350px] gap-6">
                    <div className="grid gap-2 text-center">
                        <div className="flex justify-center mb-4">
                            <div className="p-3 bg-primary/10 rounded-full">
                                <Cpu className="h-8 w-8 text-primary" />
                            </div>
                        </div>
                        <h1 className="text-3xl font-bold">Welcome back</h1>
                        <p className="text-balance text-muted-foreground">
                            Enter your credentials to access the UAIP Hub
                        </p>
                    </div>
                    <Form {...form}>
                        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                            <FormField
                                control={form.control}
                                name="email"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>Email</FormLabel>
                                        <FormControl>
                                            <Input placeholder="admin@uaip.io" {...field} />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                            <FormField
                                control={form.control}
                                name="password"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>Password</FormLabel>
                                        <div className="relative">
                                            <FormControl>
                                                <Input type="password" placeholder="••••••••" {...field} />
                                            </FormControl>
                                        </div>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                            <Button type="submit" className="w-full" disabled={isLoading}>
                                {isLoading ? (
                                    <>
                                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                        Authenticating...
                                    </>
                                ) : (
                                    "Sign in"
                                )}
                            </Button>
                        </form>
                    </Form>
                    <div className="mt-4 text-center text-sm">
                        <span className="text-muted-foreground">Don&apos;t have an account? </span>
                        <Link href="/auth/register" className="underline hover:text-primary">
                            Create an account
                        </Link>
                    </div>
                </div>
            </div>
            <div className="hidden bg-muted lg:block relative overflow-hidden">
                <div className="absolute inset-0 bg-zinc-900 border-l border-zinc-800">
                    {/* Abstract tech background */}
                    <div className="absolute top-0 right-0 -mr-20 -mt-20 w-[600px] h-[600px] bg-primary/20 rounded-full blur-3xl opacity-20 animate-pulse"></div>
                    <div className="absolute bottom-0 left-0 -ml-20 -mb-20 w-[400px] h-[400px] bg-emerald-500/20 rounded-full blur-3xl opacity-20"></div>

                    <div className="relative h-full flex flex-col items-center justify-center p-10 text-white z-10">
                        <ShieldCheck className="h-24 w-24 mb-8 text-emerald-500" />
                        <h2 className="text-4xl font-bold mb-6 text-center">Enterprise Grade Security</h2>
                        <p className="text-lg text-zinc-400 text-center max-w-md leading-relaxed">
                            Universal AI Integration Protocol provides military-grade encryption,
                            role-based access control, and complete audit trails for your
                            AI-driven infrastructure.
                        </p>

                        <div className="mt-12 grid grid-cols-2 gap-8 w-full max-w-lg">
                            <div className="bg-white/5 backdrop-blur-sm p-6 rounded-2xl border border-white/10">
                                <div className="text-3xl font-bold text-primary mb-1">99.9%</div>
                                <div className="text-sm text-zinc-400">Uptime SLA</div>
                            </div>
                            <div className="bg-white/5 backdrop-blur-sm p-6 rounded-2xl border border-white/10">
                                <div className="text-3xl font-bold text-emerald-500 mb-1">Zero</div>
                                <div className="text-sm text-zinc-400">Trust Architecture</div>
                            </div>
                        </div>
                    </div>

                    {/* Grid overlay */}
                    <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:24px_24px]"></div>
                </div>
            </div>
        </div>
    )
}
