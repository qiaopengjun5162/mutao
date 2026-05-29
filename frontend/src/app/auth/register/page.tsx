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
    <div className="bg-slate-950 min-h-[calc(100vh-3.5rem)] bg-cyber-grid animate-scanline flex items-center justify-center p-4">
      <div className="w-full max-w-sm relative z-10">
        <div className="flex items-center gap-2 mb-4 text-[10px] font-mono text-amber-500/40">
          <span className="text-amber-400">{">"}</span>
          <span>AUTH_MODULE::REGISTER</span>
          <span className="text-amber-500/20">| 0xff99...0x00</span>
        </div>

        <div className="relative bg-slate-900/50 border border-amber-500/10 clip-mech-panel p-8">
          {/* Corner crosses */}
          <div className="absolute top-1 left-1 text-amber-500/20 text-xs">+</div>
          <div className="absolute top-1 right-1 text-amber-500/20 text-xs">+</div>
          <div className="absolute bottom-1 left-1 text-amber-500/20 text-xs">+</div>
          <div className="absolute bottom-1 right-1 text-amber-500/20 text-xs">+</div>

          <h1 className="text-xl font-mono font-bold text-center mb-8 text-amber-400 tracking-wider glow-text-orange">
            用户注册
          </h1>

          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label className="text-[10px] font-mono text-slate-500 mb-1.5 block tracking-wider">USERNAME</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/80 border border-slate-700/50 focus:border-amber-500/50 text-white font-mono text-sm outline-none transition-colors placeholder:text-slate-600 clip-mech-btn"
                placeholder="输入用户名"
                required
              />
            </div>
            <div>
              <label className="text-[10px] font-mono text-slate-500 mb-1.5 block tracking-wider">PASSWORD</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/80 border border-slate-700/50 focus:border-amber-500/50 text-white font-mono text-sm outline-none transition-colors placeholder:text-slate-600 clip-mech-btn"
                placeholder="输入密码 (6+ 字符)"
                required
                minLength={6}
              />
            </div>
            <div>
              <label className="text-[10px] font-mono text-slate-500 mb-1.5 block tracking-wider">CONFIRM</label>
              <input
                type="password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                className="w-full px-4 py-3 bg-slate-950/80 border border-slate-700/50 focus:border-amber-500/50 text-white font-mono text-sm outline-none transition-colors placeholder:text-slate-600 clip-mech-btn"
                placeholder="确认密码"
                required
                minLength={6}
              />
            </div>
            {error && (
              <div className="flex items-center gap-2 text-red-400 text-xs font-mono">
                <span className="text-red-500">[ERR]</span> {error}
              </div>
            )}
            <button
              type="submit"
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-amber-500 to-orange-600 text-slate-950 font-mono font-bold text-sm tracking-wider clip-mech-btn glow-orange transition-all duration-300 disabled:opacity-50 btn-flow"
            >
              {loading ? "[CREATING...]" : "[REGISTER]"}
            </button>
          </form>

          <div className="mt-6 pt-4 border-t border-slate-800/50">
            <p className="text-center text-slate-500 text-xs font-mono">
              HAS ACCOUNT?{" "}
              <Link href="/auth/login" className="text-cyan-400 hover:text-cyan-300 transition-colors">
                LOGIN
              </Link>
            </p>
          </div>
        </div>

        <div className="mt-4 text-center text-[10px] font-mono text-slate-600">
          {"// identity.verify | blockchain.attest | keccak256"}
        </div>
      </div>
    </div>
  );
}
