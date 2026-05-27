"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api";

export default function RegisterPage() {
  const router = useRouter();
  const [form, setForm] = useState({
    username: "",
    password: "",
    confirmPassword: "",
  });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (form.username.trim().length === 0) {
      setError("用户名不能为空");
      return;
    }
    if (form.password.length < 6) {
      setError("密码长度至少 6 位");
      return;
    }
    if (form.password !== form.confirmPassword) {
      setError("两次密码不一致");
      return;
    }

    setLoading(true);
    try {
      const res = await api.register({
        username: form.username,
        password: form.password,
      });
      localStorage.setItem("token", res.token);
      localStorage.setItem("user", JSON.stringify(res));
      router.push("/items");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "注册失败");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-[calc(100vh-3.5rem)] items-center justify-center p-8">
      <div className="w-full max-w-sm border rounded-lg p-6">
        <h1 className="text-2xl font-bold text-center mb-6">注册</h1>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">用户名</label>
            <input
              className="w-full border rounded-md px-3 py-2 bg-background"
              value={form.username}
              onChange={(e) => setForm({ ...form, username: e.target.value })}
              placeholder="输入用户名"
              autoComplete="username"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">密码</label>
            <input
              type="password"
              className="w-full border rounded-md px-3 py-2 bg-background"
              value={form.password}
              onChange={(e) => setForm({ ...form, password: e.target.value })}
              placeholder="至少 6 位"
              autoComplete="new-password"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">确认密码</label>
            <input
              type="password"
              className="w-full border rounded-md px-3 py-2 bg-background"
              value={form.confirmPassword}
              onChange={(e) =>
                setForm({ ...form, confirmPassword: e.target.value })
              }
              placeholder="再次输入密码"
              autoComplete="new-password"
            />
          </div>

          {error && <p className="text-red-500 text-sm">{error}</p>}

          <button
            type="submit"
            disabled={loading}
            className="w-full inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium bg-primary text-primary-foreground shadow hover:bg-primary/90 h-10 px-4 py-2 disabled:opacity-50"
          >
            {loading ? "注册中..." : "注册"}
          </button>
        </form>

        <p className="text-center text-sm text-muted-foreground mt-4">
          已有账号？{" "}
          <Link href="/auth/login" className="text-primary hover:underline">
            登录
          </Link>
        </p>
      </div>
    </div>
  );
}
