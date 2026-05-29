"use client";

import Link from "next/link";
import { useAuth } from "@/hooks/use-auth";
import { useRouter } from "next/navigation";

export function Nav() {
  const { isLoggedIn, username, logout } = useAuth();
  const router = useRouter();

  const handleLogout = () => {
    logout();
    router.push("/");
  };

  return (
    <nav className="border-b border-cyan/10 bg-slate-950/80 backdrop-blur-md relative">
      {/* Top accent line */}
      <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-cyan/40 to-transparent" />

      <div className="container mx-auto flex h-14 items-center justify-between px-4">
        {/* Logo */}
        <Link href="/" className="text-lg font-bold font-mono tracking-wider group flex items-center gap-2">
          <span className="text-cyan group-hover:neon-cyan transition-all duration-300">木桃</span>
          <span className="text-xs text-slate-600 font-normal hidden sm:inline">MUTAO</span>
        </Link>

        {/* Nav links */}
        <div className="hidden md:flex items-center gap-1">
          {[
            { href: "/items", label: "物品", icon: "◈" },
            { href: "/items/new", label: "发布", icon: "◆" },
            { href: "/demands", label: "意向", icon: "◇" },
            { href: "/cycles", label: "交换", icon: "◉" },
          ].map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className="px-3 py-2 text-sm text-slate-400 hover:text-cyan hover:bg-cyan/5 rounded transition-all duration-300 flex items-center gap-1.5 group"
            >
              <span className="text-xs text-cyan/30 group-hover:text-cyan/70 transition-colors">{item.icon}</span>
              {item.label}
            </Link>
          ))}
        </div>

        {/* Auth section */}
        <div className="hidden md:flex items-center gap-2">
          {isLoggedIn ? (
            <>
              <span className="text-xs font-mono text-slate-500">
                <span className="text-cyan/40">USER:</span> {username}
              </span>
              <button
                onClick={handleLogout}
                className="px-3 py-1.5 text-xs font-mono text-slate-400 border border-slate-700 hover:border-red-500/50 hover:text-red-400 clip-cyber-sm transition-all duration-300"
              >
                LOGOUT
              </button>
            </>
          ) : (
            <>
              <Link
                href="/auth/login"
                className="px-3 py-1.5 text-sm text-slate-400 hover:text-cyan transition-colors"
              >
                登录
              </Link>
              <Link
                href="/auth/register"
                className="px-4 py-1.5 text-sm bg-cyan/10 border border-cyan/30 text-cyan hover:bg-cyan/20 clip-cyber-sm transition-all duration-300"
              >
                注册
              </Link>
            </>
          )}
        </div>

        {/* Mobile menu */}
        <button className="md:hidden p-2 text-slate-400 hover:text-cyan transition-colors" aria-label="菜单">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
      </div>

      {/* Bottom accent line */}
      <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-cyan/20 to-transparent" />
    </nav>
  );
}
