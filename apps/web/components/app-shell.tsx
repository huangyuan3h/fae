import Link from "next/link";
import { Bot, Settings, Sparkles } from "lucide-react";
import { cn } from "../lib/utils";

interface AppShellProps {
  title: string;
  subtitle: string;
  children: React.ReactNode;
  active: "chat" | "settings" | "login";
}

const navItems = [
  { href: "/chat", label: "Chat", icon: Bot, key: "chat" as const },
  {
    href: "/settings",
    label: "Settings",
    icon: Settings,
    key: "settings" as const
  }
];

export function AppShell({ title, subtitle, children, active }: AppShellProps) {
  return (
    <main className="mx-auto flex w-full max-w-6xl flex-col gap-6 px-4 pb-8 pt-8 sm:px-6 lg:px-8">
      <header className="rounded-2xl border border-slate-800/70 bg-slate-900/60 p-4 backdrop-blur sm:p-5">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="space-y-1">
            <p className="inline-flex items-center gap-2 rounded-full border border-sky-300/20 bg-sky-400/10 px-3 py-1 text-xs font-medium uppercase tracking-[0.16em] text-sky-200">
              <Sparkles className="h-3.5 w-3.5" />
              Local AI Workspace
            </p>
            <h1 className="text-2xl font-semibold text-slate-100 sm:text-3xl">
              {title}
            </h1>
            <p className="text-sm text-slate-400">{subtitle}</p>
          </div>
          <nav className="flex items-center gap-2">
            {navItems.map((item) => {
              const Icon = item.icon;
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={cn(
                    "inline-flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-medium transition",
                    active === item.key
                      ? "bg-sky-400/15 text-sky-100 ring-1 ring-sky-300/35"
                      : "text-slate-300 hover:bg-slate-800/80 hover:text-slate-100"
                  )}
                >
                  <Icon className="h-4 w-4" />
                  {item.label}
                </Link>
              );
            })}
          </nav>
        </div>
      </header>
      {children}
    </main>
  );
}
