"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";

export default function NewItemPage() {
  const router = useRouter();
  const [form, setForm] = useState({
    title: "",
    description: "",
    tags: "",
    value_tier: 3,
  });
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    const tags = form.tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    if (!form.title.trim()) {
      setError("标题不能为空");
      return;
    }
    if (tags.length === 0) {
      setError("至少输入一个标签");
      return;
    }

    setSubmitting(true);
    try {
      await api.createItem({
        owner_id: "00000000-0000-0000-0000-000000000001",
        title: form.title,
        description: form.description || undefined,
        tags,
        value_tier: form.value_tier,
      });
      router.push("/items");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "创建失败");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="container mx-auto max-w-lg p-8">
      <h1 className="text-2xl font-bold mb-6">发布物品</h1>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">标题</label>
          <input
            className="w-full border rounded-md px-3 py-2 bg-background"
            value={form.title}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
            placeholder="物品名称"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">描述</label>
          <textarea
            className="w-full border rounded-md px-3 py-2 bg-background"
            rows={3}
            value={form.description}
            onChange={(e) =>
              setForm({ ...form, description: e.target.value })
            }
            placeholder="物品描述（可选）"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">
            标签（逗号分隔）
          </label>
          <input
            className="w-full border rounded-md px-3 py-2 bg-background"
            value={form.tags}
            onChange={(e) => setForm({ ...form, tags: e.target.value })}
            placeholder="键盘, 外设, 电子产品"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">
            价值等级: {form.value_tier}
          </label>
          <input
            type="range"
            min={1}
            max={5}
            value={form.value_tier}
            onChange={(e) =>
              setForm({ ...form, value_tier: Number(e.target.value) })
            }
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>低</span>
            <span>高</span>
          </div>
        </div>

        {error && <p className="text-red-500 text-sm">{error}</p>}

        <Button type="submit" className="w-full" disabled={submitting}>
          {submitting ? "提交中..." : "发布"}
        </Button>
      </form>
    </div>
  );
}
