"use client";

import { useState } from "react";
import Link from "next/link";
import { useWs } from "@/hooks/use-ws";

const navLinks = [
  { href: "/items", label: "物品" },
  { href: "/items/new", label: "发布" },
  { href: "/demands", label: "意向" },
  { href: "/cycles", label: "交换" },
];

export function Nav() {
  const [open, setOpen] = useState(false);
  const { notifications, connected } = useWs();
  const unread = notifications.length;

  return (
    <nav className="border-b bg-background">
      <div className="container mx-auto flex h-14 items-center justify-between px-4">
        <Link href="/" className="text-lg font-bold">
          木桃 Mutao
        </Link>

        {/* WebSocket 连接状态 + 通知计数 */}
        <div className="hidden md:flex items-center gap-2 text-xs text-muted-foreground">
          <span className={`inline-block w-2 h-2 rounded-full ${connected ? "bg-green-500" : "bg-gray-400"}`} />
          {unread > 0 && (
            <span className="bg-primary text-primary-foreground rounded-full px-1.5 py-0.5 text-[10px]">
              {unread}
            </span>
          )}
        </div>

        {/* 桌面端导航 */}
        <div className="hidden md:flex items-center gap-2">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="px-3 py-2 text-sm rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
            >
              {link.label}
            </Link>
          ))}
        </div>

        <div className="hidden md:flex items-center gap-2">
          <Link
            href="/auth/login"
            className="px-3 py-2 text-sm rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
          >
            登录
          </Link>
          <Link
            href="/auth/register"
            className="px-3 py-2 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            注册
          </Link>
        </div>

        {/* 移动端汉堡按钮 */}
        <button
          className="md:hidden p-2 rounded-md hover:bg-accent"
          onClick={() => setOpen(!open)}
          aria-label="菜单"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            {open ? (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            ) : (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            )}
          </svg>
        </button>
      </div>

      {/* 移动端展开菜单 */}
      {open && (
        <div className="md:hidden border-t px-4 py-3 space-y-1">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              onClick={() => setOpen(false)}
              className="block px-3 py-2 text-sm rounded-md hover:bg-accent transition-colors"
            >
              {link.label}
            </Link>
          ))}
          <hr className="my-2" />
          <Link
            href="/auth/login"
            onClick={() => setOpen(false)}
            className="block px-3 py-2 text-sm rounded-md hover:bg-accent transition-colors"
          >
            登录
          </Link>
          <Link
            href="/auth/register"
            onClick={() => setOpen(false)}
            className="block px-3 py-2 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            注册
          </Link>
        </div>
      )}
    </nav>
  );
}
