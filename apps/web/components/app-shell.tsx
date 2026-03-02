"use client";

import Link from "next/link";
import { useState } from "react";
import {
  Boxes,
  Bot,
  Building2,
  ChevronsLeft,
  ChevronsRight,
  ChevronDown,
  PanelLeft,
  Puzzle,
  SlidersHorizontal,
  Settings,
  User
} from "lucide-react";
import { cn } from "../lib/utils";

interface AppShellProps {
  title: string;
  subtitle?: string;
  children: React.ReactNode;
  active: "chat" | "settings" | "providers" | "employees" | "channels" | "skills" | "login";
}

const navItems = [
  { href: "/chat", label: "Chat", icon: Bot, key: "chat" as const },
  { href: "/channels", label: "Channels", icon: Boxes, key: "channels" as const },
  {
    href: "/employees",
    label: "Employees",
    icon: Building2,
    key: "employees" as const
  },
  {
    href: "/providers",
    label: "Providers",
    icon: SlidersHorizontal,
    key: "providers" as const
  },
  {
    href: "/skills",
    label: "Skills",
    icon: Puzzle,
    key: "skills" as const
  },
  {
    href: "/settings",
    label: "Settings",
    icon: Settings,
    key: "settings" as const
  }
];

export function AppShell({ title, subtitle, children, active }: AppShellProps) {
  const [desktopCollapsed, setDesktopCollapsed] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <main className="flex min-h-screen w-full bg-transparent text-slate-900">
      {mobileOpen ? (
        <button
          type="button"
          className="fixed inset-0 z-30 bg-blue-900/20 sm:hidden"
          onClick={() => setMobileOpen(false)}
          aria-label="Close sidebar"
        />
      ) : null}

      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-40 flex w-64 flex-col border-r border-slate-200 bg-white transition-[transform,width] duration-300 ease-in-out sm:sticky sm:top-0 sm:h-screen sm:translate-x-0",
          mobileOpen ? "translate-x-0" : "-translate-x-full",
          desktopCollapsed ? "sm:w-16" : "sm:w-64"
        )}
      >
        <div className="flex h-16 items-center border-b border-slate-200 px-3">
          <Link
            href="/chat"
            className={cn(
              "inline-flex items-center gap-2 text-sm font-semibold text-slate-900",
              desktopCollapsed && "sm:mx-auto"
            )}
          >
            <span className="inline-flex h-8 w-8 items-center justify-center rounded-lg bg-blue-600 text-sm font-bold text-white">
              F
            </span>
            <span
              className={cn(
                "origin-left transition-all duration-200 ease-out",
                desktopCollapsed
                  ? "sm:w-0 sm:-translate-x-1 sm:opacity-0 sm:pointer-events-none"
                  : "sm:w-auto sm:translate-x-0 sm:opacity-100"
              )}
            >
              Fae
            </span>
          </Link>
          <button
            type="button"
            className={cn(
              "hidden rounded-lg p-1.5 text-slate-500 transition hover:bg-slate-100 sm:inline-flex",
              desktopCollapsed ? "sm:absolute sm:right-3" : "ml-auto"
            )}
            onClick={() => setDesktopCollapsed((value) => !value)}
            aria-label={desktopCollapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            {desktopCollapsed ? (
              <ChevronsRight className="h-4 w-4" />
            ) : (
              <ChevronsLeft className="h-4 w-4" />
            )}
          </button>
        </div>

        <nav className="grid gap-1 p-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <Link
                key={item.href}
                href={item.href}
                onClick={() => setMobileOpen(false)}
                className={cn(
                  "inline-flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-all duration-200",
                  active === item.key
                    ? "bg-blue-50 text-blue-700"
                    : "text-slate-600 hover:bg-blue-50 hover:text-slate-900",
                  desktopCollapsed && "sm:justify-center sm:px-2"
                )}
              >
                <Icon className="h-4 w-4" />
                <span
                  className={cn(
                    "origin-left transition-all duration-200 ease-out",
                    desktopCollapsed
                      ? "sm:w-0 sm:-translate-x-1 sm:opacity-0 sm:pointer-events-none"
                      : "sm:w-auto sm:translate-x-0 sm:opacity-100"
                  )}
                >
                  {item.label}
                </span>
              </Link>
            );
          })}
        </nav>

        <div className="mt-auto border-t border-slate-200 p-2">
          <button
            type="button"
            className={cn(
              "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-sm text-slate-600 hover:bg-blue-50",
              desktopCollapsed && "sm:justify-center"
            )}
          >
            <span className="inline-flex h-7 w-7 items-center justify-center rounded-full bg-blue-100 text-xs font-semibold text-blue-700">
              <User className="h-4 w-4" />
            </span>
            <span
              className={cn(
                "origin-left transition-all duration-200 ease-out",
                desktopCollapsed
                  ? "sm:w-0 sm:-translate-x-1 sm:opacity-0 sm:pointer-events-none"
                  : "sm:w-auto sm:translate-x-0 sm:opacity-100"
              )}
            >
              Developer
            </span>
            <ChevronDown
              className={cn(
                "ml-auto h-4 w-4 transition-opacity duration-150",
                desktopCollapsed && "sm:opacity-0 sm:pointer-events-none"
              )}
            />
          </button>
        </div>
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-20 h-16 border-b border-slate-200 bg-white/95 backdrop-blur">
          <div className="flex h-full items-center gap-3 px-4 sm:px-6">
            <button
              type="button"
              className="inline-flex rounded-lg border border-slate-300 p-2 text-slate-600 transition hover:bg-slate-100 sm:hidden"
              onClick={() => setMobileOpen(true)}
              aria-label="Open sidebar"
            >
              <PanelLeft className="h-4 w-4" />
            </button>
            <div className="min-w-0">
              <h1 className="truncate text-xl font-semibold text-slate-900">{title}</h1>
            </div>
          </div>
        </header>

        <section className="flex-1 p-4 sm:p-6">
          {subtitle ? (
            <div className="mb-4">
              <p className="text-sm text-slate-500">{subtitle}</p>
            </div>
          ) : null}
          <div className="h-full">
            {children}
          </div>
        </section>

      </section>
    </main>
  );
}
