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
    <nav className="border-b border-white/10 bg-black/40 backdrop-blur-sm">
      <div className="container mx-auto flex h-14 items-center justify-between px-4">
        <Link href="/" className="text-lg font-bold text-amber-400">
          木桃
        </Link>
        <div className="hidden md:flex items-center gap-1">
          <Link
            href="/items"
            className="px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-white/10 rounded-md transition-colors"
          >
            物品
          </Link>
          <Link
            href="/items/new"
            className="px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-white/10 rounded-md transition-colors"
          >
            发布
          </Link>
          <Link
            href="/demands"
            className="px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-white/10 rounded-md transition-colors"
          >
            意向
          </Link>
          <Link
            href="/cycles"
            className="px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-white/10 rounded-md transition-colors"
          >
            交换
          </Link>
        </div>
        <div className="hidden md:flex items-center gap-2">
          {isLoggedIn ? (
            <>
              <span className="text-sm text-gray-400">{username}</span>
              <button
                onClick={handleLogout}
                className="px-3 py-1.5 text-sm text-gray-300 hover:text-white border border-white/20 hover:border-white/40 rounded-md transition-colors"
              >
                退出
              </button>
            </>
          ) : (
            <>
              <Link
                href="/auth/login"
                className="px-3 py-1.5 text-sm text-gray-300 hover:text-white transition-colors"
              >
                登录
              </Link>
              <Link
                href="/auth/register"
                className="px-3 py-1.5 text-sm bg-amber-500 hover:bg-amber-600 text-black font-medium rounded-md transition-colors"
              >
                注册
              </Link>
            </>
          )}
        </div>
      </div>
    </nav>
  );
}
