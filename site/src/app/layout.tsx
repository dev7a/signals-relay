import type { Metadata } from "next";
import { IBM_Plex_Mono, Manrope } from "next/font/google";
import type { ReactNode } from "react";
import { Provider } from "@/components/provider";
import { siteOrigin } from "@/lib/shared";
import "./global.css";

const sans = Manrope({
  subsets: ["latin"],
  variable: "--font-sans",
});

const mono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: ["400", "500"],
  variable: "--font-mono",
});

export const metadata: Metadata = {
  title: {
    default: "Signals Relay Docs",
    template: "%s | Signals Relay Docs",
  },
  description: "Documentation for the Signals Relay AWS serverless OTLP export pipeline.",
  metadataBase: new URL(siteOrigin),
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className={`${sans.variable} ${mono.variable}`} suppressHydrationWarning>
      <body className="min-h-screen bg-fd-background text-fd-foreground antialiased">
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}
