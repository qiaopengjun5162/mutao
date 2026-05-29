"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/hooks/use-auth";
import { register as apiRegister } from "@/lib/api";

export default function RegisterPage() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const { login } = useAuth();
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (password !== confirm) {
      setError("两次密码不一致");
      return;
    }
    setLoading(true);
    try {
      const res = await apiRegister({ username, password });
      login(res.token, res.user_id, res.username);
      router.push("/items");
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "注册失败");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="grid-bg scanline min-h-[calc(100vh-3.5rem)] flex items-center justify-center p-4">
      <div className="w-full max-w-sm relative">
        {/* Terminal header */}
        <div className="flex items-center gap-2 mb-4 text-xs font-mono text-slate-500">
          <span className="text-orange/50">{">"}</span>
          <span>AUTH_MODULE::REGISTER</span>
        </div>

        {/* Card */}
        <div className="relative bg-slate-900/50 border border-orange/10 clip-cyber p-8">
          {/* Corner decorations */}
          <div className="absolute top-0 left-0 w-3 h-3 border-t border-l border-orange/40" />
          <div className="absolute top-0 right-0 w-3 h-3 border-t border-r border-orange/40" />
          <div className="absolute bottom-0 left-0 w-3 h-3 border-b border-l border-orange/40" />
          <div className="absolute bottom-0 right-0 w-3 h-3 border-b border-r border-orange/40" />

          <h1 className="text-xl font-mono font-bold text-center mb-8 text-orange tracking-wider">
            用户注册
          </h1>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="text-xs font-mono text-slate-500 mb-1 block">USERNAME</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700 focus:border-orange/50 text-white font-mono text-sm rounded-none outline-none transition-colors placeholder:text-slate-600"
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
                className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700 focus:border-orange/50 text-white font-mono text-sm rounded-none outline-none transition-colors placeholder:text-slate-600"
                placeholder="输入密码 (6+ 字符)"
                required
                minLength={6}
              />
            </div>
            <div>
              <label className="text-xs font-mono text-slate-500 mb-1 block">CONFIRM</label>
              <input
                type="password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700 focus:border-orange/50 text-white font-mono text-sm rounded-none outline-none transition-colors placeholder:text-slate-600"
                placeholder="确认密码"
                required
                minLength={6}
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
              className="w-full py-3 bg-orange text-slate-950 font-mono font-bold text-sm tracking-wider clip-cyber-sm glow-orange transition-all duration-300 disabled:opacity-50 btn-flow relative overflow-hidden"
            >
              {loading ? "CREATING..." : "REGISTER"}
            </button>
          </form>

          <div className="mt-6 pt-4 border-t border-slate-800">
            <p className="text-center text-slate-500 text-xs font-mono">
              HAS ACCOUNT?{" "}
              <Link href="/auth/login" className="text-orange hover:text-orange/80 transition-colors">
                LOGIN
              </Link>
            </p>
          </div>
        </div>

        {/* Bottom decoration */}
        <div className="mt-4 text-center text-xs font-mono text-slate-600">
          {"// identity.verify | blockchain.attest"}
        </div>
      </div>
    </div>
  );
}
