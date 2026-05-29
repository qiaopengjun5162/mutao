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
    <nav className="backdrop-blur-md bg-slate-900/60 border-b border-cyan-500/20 sticky top-0 z-50">
      <div className="container mx-auto flex h-14 items-center justify-between px-4">
        {/* Logo */}
        <Link href="/" className="text-lg font-black tracking-widest group flex items-center gap-2">
          <span className="bg-gradient-to-r from-cyan-400 to-amber-400 bg-clip-text text-transparent group-hover:glow-text-cyan transition-all duration-300">
            木桃
          </span>
          <span className="text-[10px] text-cyan-500/30 font-mono hidden sm:inline">v0.1</span>
        </Link>

        {/* Nav links */}
        <div className="hidden md:flex items-center gap-1">
          {[
            { href: "/items", label: "物品" },
            { href: "/items/new", label: "发布" },
            { href: "/demands", label: "意向" },
            { href: "/cycles", label: "交换" },
          ].map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className="px-3 py-2 text-sm text-slate-400 hover:text-cyan-400 hover:glow-text-cyan transition-all duration-300"
            >
              {item.label}
            </Link>
          ))}
        </div>

        {/* Auth */}
        <div className="hidden md:flex items-center gap-3">
          {isLoggedIn ? (
            <>
              <span className="text-[10px] font-mono text-slate-500">
                <span className="text-cyan-500/50">USER:</span> {username}
              </span>
              <button
                onClick={handleLogout}
                className="clip-mech-btn px-3 py-1.5 text-xs font-mono text-slate-400 border border-slate-700 hover:border-red-500/50 hover:text-red-400 transition-all duration-300"
              >
                LOGOUT
              </button>
            </>
          ) : (
            <>
              <Link
                href="/auth/login"
                className="px-3 py-1.5 text-sm text-slate-400 hover:text-cyan-400 transition-colors"
              >
                登录
              </Link>
              <Link
                href="/auth/register"
                className="clip-mech-btn px-4 py-1.5 text-sm font-bold bg-amber-500 text-slate-950 hover:bg-amber-400 transition-colors"
              >
                注册
              </Link>
            </>
          )}
        </div>

        {/* Mobile */}
        <button className="md:hidden p-2 text-slate-400 hover:text-cyan-400 transition-colors" aria-label="菜单">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
      </div>
    </nav>
  );
}
