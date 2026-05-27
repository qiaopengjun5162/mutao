"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { api, type Item } from "@/lib/api";

const statusLabel: Record<string, string> = {
  Idle: "空闲",
  Matching: "匹配中",
  Completed: "已完成",
  Archived: "已归档",
};

const tierLabel = ["", "低", "中低", "中", "中高", "高"];

export default function ItemsPage() {
  const [items, setItems] = useState<Item[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    api
      .listItems()
      .then(setItems)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="container mx-auto p-8">加载中...</div>;
  if (error)
    return <div className="container mx-auto p-8 text-red-500">{error}</div>;

  return (
    <div className="container mx-auto p-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">物品列表</h1>
        <Link
          href="/items/new"
          className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2"
        >
          发布物品
        </Link>
      </div>

      {items.length === 0 ? (
        <p className="text-muted-foreground">暂无物品</p>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {items.map((item) => (
            <Link
              key={item.id}
              href={`/items/${item.id}`}
              className="border rounded-lg p-4 hover:bg-accent transition-colors"
            >
              <div className="flex items-center justify-between mb-2">
                <h2 className="font-semibold">{item.title}</h2>
                <span className="text-xs px-2 py-1 rounded bg-secondary">
                  {statusLabel[item.status] || item.status}
                </span>
              </div>
              <p className="text-sm text-muted-foreground mb-2 line-clamp-2">
                {item.description || "无描述"}
              </p>
              <div className="flex items-center justify-between">
                <div className="flex gap-1">
                  {item.tags.map((tag) => (
                    <span
                      key={tag}
                      className="text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
                <span className="text-xs text-muted-foreground">
                  价值: {tierLabel[item.value_tier] || item.value_tier}
                </span>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
