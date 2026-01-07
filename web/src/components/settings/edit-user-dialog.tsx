"use client"

import { useState, useEffect } from "react"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import * as z from "zod"
import { Loader2, Wand2, Shield, AlertTriangle } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog"
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
    FormDescription,
} from "@/components/ui/form"
import { Input } from "@/components/ui/input"
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { Checkbox } from "@/components/ui/checkbox"
import { Separator } from "@/components/ui/separator"
import { toast } from "sonner"
import api from "@/lib/api"

const formSchema = z.object({
    name: z.string().min(2, "Name must be at least 2 characters"),
    role: z.string().min(1, "Please select a role"),
})

const passwordSchema = z.object({
    newPassword: z.string()
        .min(12, "Password must be at least 12 characters")
        .regex(/[A-Z]/, "Must contain an uppercase letter")
        .regex(/[a-z]/, "Must contain a lowercase letter")
        .regex(/[0-9]/, "Must contain a number")
        .regex(/[^A-Za-z0-9]/, "Must contain a special character"),
    requireChange: z.boolean(),
})

interface EditUserDialogProps {
    user: {
        id: string
        name: string
        email: string
        role: string
    } | null
    open: boolean
    onOpenChange: (open: boolean) => void
    onSuccess: () => void
}

export function EditUserDialog({ user, open, onOpenChange, onSuccess }: EditUserDialogProps) {
    const [isLoading, setIsLoading] = useState(false)
    const [isPasswordLoading, setIsPasswordLoading] = useState(false)
    const [showPasswordReset, setShowPasswordReset] = useState(false)

    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            name: "",
            role: "viewer",
        },
    })

    const passwordForm = useForm<z.infer<typeof passwordSchema>>({
        resolver: zodResolver(passwordSchema),
        defaultValues: {
            newPassword: "",
            requireChange: true,
        },
    })

    useEffect(() => {
        if (user) {
            form.reset({
                name: user.name,
                role: user.role,
            })
            setShowPasswordReset(false)
            passwordForm.reset()
        }
    }, [user, form, passwordForm])

    async function onSubmit(values: z.infer<typeof formSchema>) {
        if (!user) return
        setIsLoading(true)
        try {
            await api.put(`/api/v1/users/${user.id}`, values)
            toast.success("User updated successfully")
            onOpenChange(false)
            onSuccess()
        } catch (error: any) {
            console.error(error)
            toast.error(error.response?.data?.message || "Failed to update user")
        } finally {
            setIsLoading(false)
        }
    }

    const generatePassword = () => {
        const length = 16
        const charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"
        let retVal = ""
        for (let i = 0, n = charset.length; i < length; ++i) {
            retVal += charset.charAt(Math.floor(Math.random() * n))
        }
        passwordForm.setValue("newPassword", retVal)
        passwordForm.trigger("newPassword")
        toast.info("Secure password generated")
    }

    async function onPasswordSubmit(values: z.infer<typeof passwordSchema>) {
        if (!user) return
        if (!confirm("Are you sure you want to reset this user's password? They will be logged out on all devices.")) return

        setIsPasswordLoading(true)
        try {
            await api.put(`/api/v1/users/${user.id}/password`, {
                new_password: values.newPassword,
                require_change: values.requireChange
            })
            toast.success("Password reset successfully")
            setShowPasswordReset(false)
            passwordForm.reset()
        } catch (error: any) {
            console.error(error)
            toast.error(error.response?.data?.message || "Failed to reset password")
        } finally {
            setIsPasswordLoading(false)
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[500px]">
                <DialogHeader>
                    <DialogTitle>Edit User: {user?.email}</DialogTitle>
                    <DialogDescription>
                        Update user details or reset their password.
                    </DialogDescription>
                </DialogHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                        <FormField
                            control={form.control}
                            name="name"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>Full Name</FormLabel>
                                    <FormControl>
                                        <Input {...field} />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                        <FormField
                            control={form.control}
                            name="role"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>Role</FormLabel>
                                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="Select a role" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                            <SelectItem value="viewer">Viewer (Read Only)</SelectItem>
                                            <SelectItem value="operator">Operator (Manage Devices)</SelectItem>
                                            <SelectItem value="admin">Administrator (Full Access)</SelectItem>
                                        </SelectContent>
                                    </Select>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                        <div className="flex justify-end">
                            <Button type="submit" disabled={isLoading}>
                                {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                Save Changes
                            </Button>
                        </div>
                    </form>
                </Form>

                <Separator className="my-4" />

                <div className="space-y-4">
                    <div className="flex items-center justify-between">
                        <div className="space-y-1">
                            <h4 className="text-sm font-medium flex items-center gap-2">
                                <Shield className="h-4 w-4 text-orange-500" />
                                Security Zone
                            </h4>
                            <p className="text-sm text-muted-foreground">
                                Reset user password if they forgot it.
                            </p>
                        </div>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setShowPasswordReset(!showPasswordReset)}
                        >
                            {showPasswordReset ? "Cancel Reset" : "Reset Password"}
                        </Button>
                    </div>

                    {showPasswordReset && (
                        <Form {...passwordForm}>
                            <form onSubmit={passwordForm.handleSubmit(onPasswordSubmit)} className="space-y-4 border p-4 rounded-md bg-muted/50">
                                <FormField
                                    control={passwordForm.control}
                                    name="newPassword"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>New Password</FormLabel>
                                            <div className="flex gap-2">
                                                <FormControl>
                                                    <Input type="text" {...field} placeholder="Generate secure password" />
                                                </FormControl>
                                                <Button
                                                    type="button"
                                                    variant="outline"
                                                    size="icon"
                                                    onClick={generatePassword}
                                                    title="Generate"
                                                >
                                                    <Wand2 className="h-4 w-4" />
                                                </Button>
                                                <Button
                                                    type="button"
                                                    variant="outline"
                                                    size="icon"
                                                    onClick={() => {
                                                        navigator.clipboard.writeText(passwordForm.getValues("newPassword"))
                                                        toast.success("Password copied")
                                                    }}
                                                    title="Copy"
                                                    disabled={!passwordForm.getValues("newPassword")}
                                                >
                                                    <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" className="h-4 w-4">
                                                        <path d="M1 9.50006C1 10.3285 1.67157 11.0001 2.5 11.0001H4L4 10.0001H2.5C2.22386 10.0001 2 9.7762 2 9.50006L2 2.50006C2 2.22392 2.22386 2.00006 2.5 2.00006L9.5 2.00006C9.77614 2.00006 10 2.22392 10 2.50006V4.00006H11V2.50006C11 1.67163 10.3284 1.00006 9.5 1.00006L2.5 1.00006C1.67157 1.00006 1 1.67163 1 2.50006V9.50006ZM5 5.50006C5 4.67163 5.67157 4.00006 6.5 4.00006H12.5C13.3284 4.00006 14 4.67163 14 5.50006V12.5001C14 13.3285 13.3284 14.0001 12.5 14.0001H6.5C5.67157 14.0001 5 13.3285 5 12.5001V5.50006ZM6.5 5.00006H12.5C12.7761 5.00006 13 5.22392 13 5.50006V12.5001C13 12.7762 12.7761 13.0001 12.5 13.0001H6.5C6.22386 13.0001 6 12.7762 6 12.5001V5.50006C6 5.22392 6.22386 5.00006 6.5 5.00006Z" fill="currentColor" fillRule="evenodd" clipRule="evenodd"></path>
                                                    </svg>
                                                </Button>
                                            </div>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                                <FormField
                                    control={passwordForm.control}
                                    name="requireChange"
                                    render={({ field }) => (
                                        <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                                            <FormControl>
                                                <Checkbox
                                                    checked={field.value}
                                                    onCheckedChange={field.onChange}
                                                />
                                            </FormControl>
                                            <div className="space-y-1 leading-none">
                                                <FormLabel>
                                                    Require Password Change
                                                </FormLabel>
                                                <FormDescription>
                                                    User will be forced to change this password on next login.
                                                </FormDescription>
                                            </div>
                                        </FormItem>
                                    )}
                                />
                                <Button type="submit" variant="destructive" className="w-full" disabled={isPasswordLoading}>
                                    {isPasswordLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                    <AlertTriangle className="mr-2 h-4 w-4" />
                                    Confirm Reset Password
                                </Button>
                            </form>
                        </Form>
                    )}
                </div>
            </DialogContent>
        </Dialog>
    )
}
