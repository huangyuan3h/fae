import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "fae",
  description: "Local-first AI assistant daemon UI"
};

export default function RootLayout(props: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{props.children}</body>
    </html>
  );
}
