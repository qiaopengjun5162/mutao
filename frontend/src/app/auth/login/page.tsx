"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/hooks/use-auth";
import { login as apiLogin } from "@/lib/api";

export default function LoginPage() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const { login } = useAuth();
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      const res = await apiLogin({ username, password });
      login(res.token, res.user_id, res.username);
      router.push("/items");
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "登录失败");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid-bg scanline min-h-[calc(100vh-3.5rem)] flex items-center justify-center p-4">
      <div className="w-full max-w-sm relative">
        {/* Terminal header */}
        <div className="flex items-center gap-2 mb-4 text-xs font-mono text-slate-500">
          <span className="text-cyan/50">{">"}</span>
          <span>AUTH_MODULE::LOGIN</span>
        </div>

        {/* Card */}
        <div className="relative bg-slate-900/50 border border-cyan/10 clip-cyber p-8">
          {/* Corner decorations */}
          <div className="absolute top-0 left-0 w-3 h-3 border-t border-l border-cyan/40" />
          <div className="absolute top-0 right-0 w-3 h-3 border-t border-r border-cyan/40" />
          <div className="absolute bottom-0 left-0 w-3 h-3 border-b border-l border-cyan/40" />
          <div className="absolute bottom-0 right-0 w-3 h-3 border-b border-r border-cyan/40" />

          <h1 className="text-xl font-mono font-bold text-center mb-8 text-cyan tracking-wider">
            用户登录
          </h1>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="text-xs font-mono text-slate-500 mb-1 block">USERNAME</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700 focus:border-cyan/50 text-white font-mono text-sm rounded-none outline-none transition-colors placeholder:text-slate-600"
                placeholder="输入用户名"
                required
              />
            </div>
            <div>
              <label className="text-xs font-mono text-slate-500 mb-1 block">PASSWORD</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700 focus:border-cyan/50 text-white font-mono text-sm rounded-none outline-none transition-colors placeholder:text-slate-600"
                placeholder="输入密码"
                required
              />
            </div>
            {error && (
              <div className="flex items-center gap-2 text-red-400 text-xs font-mono">
                <span className="text-red-500">!</span> {error}
              </div>
            )}
            <button
              type="submit"
              disabled={loading}
              className="w-full py-3 bg-cyan text-slate-950 font-mono font-bold text-sm tracking-wider clip-cyber-sm glow-cyan transition-all duration-300 disabled:opacity-50 btn-flow relative overflow-hidden"
            >
              {loading ? "AUTHENTICATING..." : "LOGIN"}
            </button>
          </form>

          <div className="mt-6 pt-4 border-t border-slate-800">
            <p className="text-center text-slate-500 text-xs font-mono">
              NO ACCOUNT?{" "}
              <Link href="/auth/register" className="text-cyan hover:text-cyan/80 transition-colors">
                REGISTER
              </Link>
            </p>
          </div>
        </div>

        {/* Bottom decoration */}
        <div className="mt-4 text-center text-xs font-mono text-slate-600">
          {"// encrypted_connection | tls_1.3"}
        </div>
      </div>
    </div>
  );
}
