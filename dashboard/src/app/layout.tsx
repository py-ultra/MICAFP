import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/toaster";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "یونیفایدشیلد — موتور ضد سانسور هوشمند",
  description: "موتور ضد سانسور هوشمند چند هسته‌ای بهینه‌شده برای ایران — ۹ هسته، هوش مصنوعی داخلی، تعویض خودکار",
  keywords: ["UnifiedShield", "ضد سانسور", "ایران", "VPN", "هوش مصنوعی", "Hiddify", "Xray", "sing-box", "Psiphon"],
  icons: {
    icon: "/logo.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="fa" dir="rtl" suppressHydrationWarning className={`${geistSans.variable} ${geistMono.variable}`}>
      <body className="antialiased bg-background text-foreground">
        {children}
        <Toaster />
      </body>
    </html>
  );
}
