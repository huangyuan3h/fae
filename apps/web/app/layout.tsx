import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "fae",
  description: "Local-first AI assistant daemon UI"
};

export default function RootLayout(props: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="antialiased">
        <div className="pointer-events-none fixed inset-0 grid-fade" aria-hidden="true" />
        <div className="relative min-h-screen">{props.children}</div>
      </body>
    </html>
  );
}
