import type { Metadata } from "next";
import { Instrument_Serif, Inter, JetBrains_Mono } from "next/font/google";
import type { ReactNode } from "react";
import { Provider } from "@/components/provider";
import { siteOrigin, withBasePath } from "@/lib/shared";
import "./global.css";

const sans = Inter({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--font-sans",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  weight: ["400", "500"],
  variable: "--font-mono",
});

const serif = Instrument_Serif({
  subsets: ["latin"],
  weight: ["400"],
  style: ["normal", "italic"],
  variable: "--font-serif",
});

export const metadata: Metadata = {
  title: {
    default: "Signals Relay Docs",
    template: "%s | Signals Relay Docs",
  },
  description: "Documentation for the Signals Relay AWS serverless OTLP export pipeline.",
  metadataBase: new URL(siteOrigin),
  icons: {
    icon: [
      { url: withBasePath("/favicon.svg"), type: "image/svg+xml" },
      { url: withBasePath("/favicon.ico"), sizes: "any" },
      { url: withBasePath("/favicon-32.png"), sizes: "32x32", type: "image/png" },
      { url: withBasePath("/favicon-16.png"), sizes: "16x16", type: "image/png" },
    ],
    apple: [{ url: withBasePath("/apple-touch-icon.png"), sizes: "180x180", type: "image/png" }],
  },
  manifest: withBasePath("/site.webmanifest"),
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html
      className={`${sans.variable} ${mono.variable} ${serif.variable}`}
      lang="en"
      suppressHydrationWarning
    >
      <body className="min-h-screen bg-fd-background text-fd-foreground antialiased">
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}
