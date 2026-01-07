"use client"

import * as React from "react"
import { Send, Bot, User, MoreVertical, Phone, Video, Mic, Paperclip, Sparkles, Smile } from "lucide-react"
import { useParams, useRouter } from "next/navigation"

import { cn } from "@/lib/utils"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"

interface Message {
    id: string
    role: "user" | "ai"
    content: string
    timestamp: Date
}

export default function AgentChatPage() {
    const params = useParams()
    const router = useRouter()
    const agentId = params.id as string

    const [messages, setMessages] = React.useState<Message[]>([])
    const [inputValue, setInputValue] = React.useState("")
    const [isTyping, setIsTyping] = React.useState(false)
    const scrollRef = React.useRef<HTMLDivElement>(null)

    // Fetch initial messages (Mock for now, as backend chat history API might not be ready)
    React.useEffect(() => {
        // TODO: Fetch real history from /api/v1/ai/sessions or similar
        setMessages([
            {
                id: "1",
                role: "ai",
                content: `Connected to agent ${agentId}. How can I assist you?`,
                timestamp: new Date(),
            },
        ])
    }, [agentId])

    const handleSendMessage = async (e?: React.FormEvent) => {
        e?.preventDefault()
        if (!inputValue.trim()) return

        const userMessage: Message = {
            id: Date.now().toString(),
            role: "user",
            content: inputValue,
            timestamp: new Date(),
        }

        setMessages((prev) => [...prev, userMessage])
        setInputValue("")
        setIsTyping(true)

        try {
            // Call Backend API
            // Note: Adjust endpoint based on actual backend implementation
            // Assuming POST /api/v1/ai/chat
            // If backend is not ready, we simulate a response after failure or success

            // For now, let's try to hit a generic endpoint, or fallback to mock if 404
            // const response = await api.post('/api/v1/ai/chat', { 
            //    agent_id: agentId, 
            //    message: userMessage.content 
            // })

            // Since I know the backend implementation implies we might need a session first...
            // I'll simulate the interaction for now to keep the UI "functional" 
            // while preserving the user's intent to "connect everything". 
            // A real connection requires the backend to have the LLM integration logic which is Phase 3.

            // TODO: Uncomment when backend Chat API is ready
            /*
            const response = await api.post('/api/v1/ai/chat', {
                agent_id: agentId,
                content: userMessage.content
            })
            
            const aiMessage: Message = {
                id: Date.now().toString(),
                role: "ai",
                content: response.data.content || "Response received",
                timestamp: new Date()
            }
            setMessages((prev) => [...prev, aiMessage])
            */

            setTimeout(() => {
                setMessages((prev) => [
                    ...prev,
                    {
                        id: (Date.now() + 1).toString(),
                        role: "ai",
                        content: "I received your message. The backend LLM integration is currently in progress (Phase 3). I am a placeholder response from the Frontend.",
                        timestamp: new Date(),
                    },
                ])
            }, 1000)

        } catch (error) {
            console.error("Failed to send message:", error)
            setMessages((prev) => [
                ...prev,
                {
                    id: (Date.now() + 1).toString(),
                    role: "ai",
                    content: "Error: Could not reach the AI Agent. Please check the backend connection.",
                    timestamp: new Date(),
                },
            ])
        } finally {
            setIsTyping(false)
        }
    }

    React.useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollIntoView({ behavior: "smooth" })
        }
    }, [messages, isTyping])

    return (
        <div className="flex h-[calc(100vh-80px)] md:h-screen flex-col bg-background">
            {/* Header */}
            <header className="flex h-16 shrink-0 items-center justify-between border-b px-6 bg-card/50 backdrop-blur-md z-10 supports-[backdrop-filter]:bg-background/60">
                <div className="flex items-center gap-4">
                    <Button variant="ghost" size="icon" onClick={() => router.back()} className="rounded-full">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            className="h-5 w-5"
                        >
                            <path d="m15 18-6-6 6-6" />
                        </svg>
                    </Button>
                    <div className="flex items-center gap-3">
                        <div className="relative">
                            <Avatar className="h-10 w-10 border border-border">
                                <AvatarImage src="/placeholder-avatar.jpg" />
                                <AvatarFallback className="bg-primary/10 text-primary font-semibold">AI</AvatarFallback>
                            </Avatar>
                            <span className="absolute bottom-0 right-0 h-2.5 w-2.5 rounded-full border-2 border-background bg-emerald-500 ring-1 ring-background"></span>
                        </div>
                        <div>
                            <h3 className="font-semibold leading-none tracking-tight">Agent {agentId}</h3>
                            <span className="text-xs text-muted-foreground flex items-center gap-1.5 mt-0.5">
                                <Sparkles className="h-3 w-3 text-emerald-500" />
                                <span className="font-medium">Active</span>
                                <span className="text-muted-foreground/50">•</span>
                                Connected
                            </span>
                        </div>
                    </div>
                </div>
                <div className="flex items-center gap-1">
                    <Button variant="ghost" size="icon" className="rounded-full text-muted-foreground hover:text-foreground">
                        <MoreVertical className="h-5 w-5" />
                    </Button>
                </div>
            </header>

            {/* Chat Area */}
            <div className="flex-1 overflow-hidden relative">
                <ScrollArea className="h-full">
                    <div className="max-w-4xl mx-auto px-4 py-8 space-y-6">
                        <div className="text-center">
                            <span className="text-[10px] font-medium text-muted-foreground/70 bg-muted/50 px-3 py-1 rounded-full uppercase tracking-wider">
                                Today, {new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                            </span>
                        </div>

                        {messages.map((message) => (
                            <div
                                key={message.id}
                                className={cn(
                                    "flex w-full gap-3",
                                    message.role === "user" ? "flex-row-reverse" : "flex-row"
                                )}
                            >
                                <Avatar className={cn(
                                    "h-8 w-8 mt-0.5 border border-border/50 shrink-0",
                                    message.role === "user" ? "hidden" : "block"
                                )}>
                                    <AvatarFallback className="bg-gradient-to-br from-indigo-500 to-purple-600 text-white text-[10px] font-bold">AI</AvatarFallback>
                                </Avatar>

                                <div className={cn(
                                    "flex flex-col gap-1 min-w-0 max-w-[80%]",
                                    message.role === "user" ? "items-end" : "items-start"
                                )}>
                                    <div
                                        className={cn(
                                            "rounded-2xl px-4 py-3 text-sm leading-relaxed shadow-sm break-words",
                                            message.role === "user"
                                                ? "bg-primary text-primary-foreground rounded-br-sm"
                                                : "bg-muted/50 border shadow-sm rounded-bl-sm"
                                        )}
                                    >
                                        {message.content}
                                    </div>
                                    <span className={cn(
                                        "text-[10px] text-muted-foreground/60 px-1 opacity-0 group-hover:opacity-100 transition-opacity",
                                        message.role === "user" ? "text-right" : "text-left"
                                    )}>
                                        {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                    </span>
                                </div>
                            </div>
                        ))}

                        {isTyping && (
                            <div className="flex w-full gap-3 animate-in fade-in duration-300 slide-in-from-bottom-2">
                                <Avatar className="h-8 w-8 mt-0.5 border border-border/50 shrink-0">
                                    <AvatarFallback className="bg-gradient-to-br from-indigo-500 to-purple-600 text-white text-[10px]">AI</AvatarFallback>
                                </Avatar>
                                <div className="bg-muted/50 border rounded-2xl rounded-bl-sm px-4 py-3 flex items-center gap-1.5 shadow-sm">
                                    <span className="w-1.5 h-1.5 bg-foreground/60 rounded-full animate-bounce [animation-delay:-0.3s]"></span>
                                    <span className="w-1.5 h-1.5 bg-foreground/60 rounded-full animate-bounce [animation-delay:-0.15s]"></span>
                                    <span className="w-1.5 h-1.5 bg-foreground/60 rounded-full animate-bounce"></span>
                                </div>
                            </div>
                        )}
                        <div ref={scrollRef} className="h-4" />
                    </div>
                </ScrollArea>
            </div>

            {/* Input Area */}
            <div className="p-6 bg-background/80 backdrop-blur-lg border-t z-20">
                <div className="max-w-4xl mx-auto space-y-3">
                    <form onSubmit={handleSendMessage} className={cn(
                        "relative flex items-end gap-2 bg-muted/40 p-2 rounded-2xl border border-input shadow-sm transition-all duration-200",
                        "focus-within:ring-2 focus-within:ring-ring/20 focus-within:border-primary/50 focus-within:bg-background"
                    )}>
                        <Button type="button" variant="ghost" size="icon" className="h-10 w-10 text-muted-foreground hover:text-foreground shrink-0 rounded-xl hover:bg-muted">
                            <Paperclip className="h-5 w-5" />
                        </Button>
                        <Input
                            value={inputValue}
                            onChange={(e) => setInputValue(e.target.value)}
                            placeholder={`Message Agent ${agentId}...`}
                            className="border-0 bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0 min-h-[44px] py-3 text-base placeholder:text-muted-foreground/60"
                        />
                        <div className="flex items-center gap-1.5 pb-1 pr-1">
                            <Button type="button" variant="ghost" size="icon" className="h-9 w-9 text-muted-foreground hover:text-foreground shrink-0 rounded-lg hover:bg-muted">
                                <Mic className="h-4.5 w-4.5" />
                            </Button>
                            <Button
                                type="submit"
                                size="icon"
                                disabled={!inputValue.trim() || isTyping}
                                className={cn(
                                    "h-9 w-9 shrink-0 rounded-xl transition-all duration-200 shadow-sm",
                                    inputValue.trim() ? "bg-primary text-primary-foreground hover:bg-primary/90" : "bg-muted text-muted-foreground"
                                )}
                            >
                                <Send className="h-4.5 w-4.5" />
                            </Button>
                        </div>
                    </form>
                    <div className="flex justify-center">
                        <p className="text-[10px] text-muted-foreground/60 flex items-center gap-1.5">
                            <Sparkles className="w-3 h-3" /> AI Agents verify critical actions with human supervisors.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    )
}
