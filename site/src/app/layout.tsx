import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Provider } from "@/components/provider";
import { siteOrigin } from "@/lib/shared";
import "./global.css";

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
    <html lang="en" suppressHydrationWarning>
      <body className="min-h-screen bg-fd-background text-fd-foreground antialiased">
        <Provider>{children}</Provider>
      </body>
    </html>
  );
}
