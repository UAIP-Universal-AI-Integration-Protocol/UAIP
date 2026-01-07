import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "UAIP Hub",
  description: "Universal AI Integration Protocol Hub Interface",
};

import { AppSidebar, MobileSidebar } from "@/components/app-sidebar";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased bg-background text-foreground`}
      >
        <div className="flex h-full min-h-screen">
          <div className="hidden h-full md:flex md:w-72 md:flex-col md:fixed md:inset-y-0 z-[80]">
            <AppSidebar />
          </div>
          <main className="md:pl-72 flex-1 h-full">
            <div className="flex items-center p-4 md:hidden border-b mb-4">
              <MobileSidebar />
              <span className="font-bold ml-2">UAIP Hub</span>
            </div>
            {children}
          </main>
        </div>
      </body>
    </html>
  );
}
