"use client";

import { useEffect, useState, use } from "react";
import { api, type Item, type SwapCycle } from "@/lib/api";
import { Button } from "@/components/ui/button";

const statusLabel: Record<string, string> = {
  Idle: "空闲",
  Matching: "匹配中",
  Completed: "已完成",
  Archived: "已归档",
};

export default function ItemDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const [item, setItem] = useState<Item | null>(null);
  const [cycles, setCycles] = useState<SwapCycle[]>([]);
  const [loading, setLoading] = useState(true);
  const [matching, setMatching] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    api
      .getItem(id)
      .then(setItem)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  const handleMatch = async () => {
    setMatching(true);
    setError("");
    try {
      const result = await api.matchItem(id);
      setCycles(result);
      if (result.length === 0) {
        setError("暂未找到匹配的交换环");
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "匹配失败");
    } finally {
      setMatching(false);
    }
  };

  if (loading) return <div className="container mx-auto p-8">加载中...</div>;
  if (!item)
    return <div className="container mx-auto p-8 text-red-500">{error}</div>;

  return (
    <div className="container mx-auto max-w-2xl p-8">
      <div className="border rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-bold">{item.title}</h1>
          <span className="text-sm px-3 py-1 rounded-full bg-secondary">
            {statusLabel[item.status] || item.status}
          </span>
        </div>

        <p className="text-muted-foreground mb-4">
          {item.description || "无描述"}
        </p>

        <div className="flex gap-2 mb-4">
          {item.tags.map((tag) => (
            <span
              key={tag}
              className="text-sm px-3 py-1 rounded-full bg-primary/10 text-primary"
            >
              {tag}
            </span>
          ))}
        </div>

        <div className="grid grid-cols-2 gap-4 text-sm mb-6">
          <div>
            <span className="text-muted-foreground">价值等级:</span>{" "}
            {item.value_tier}/5
          </div>
          <div>
            <span className="text-muted-foreground">物品 ID:</span>{" "}
            <code className="text-xs">{item.id}</code>
          </div>
          <div>
            <span className="text-muted-foreground">所有者:</span>{" "}
            <code className="text-xs">{item.owner_id}</code>
          </div>
          <div>
            <span className="text-muted-foreground">创建时间:</span>{" "}
            {new Date(item.created_at).toLocaleString("zh-CN")}
          </div>
        </div>

        {item.status === "Idle" && (
          <Button onClick={handleMatch} disabled={matching} className="w-full">
            {matching ? "匹配中..." : "触发匹配"}
          </Button>
        )}
      </div>

      {cycles.length > 0 && (
        <div className="mt-6">
          <h2 className="text-lg font-semibold mb-4">
            匹配到 {cycles.length} 个交换环
          </h2>
          {cycles.map((cycle) => (
            <div key={cycle.id} className="border rounded-lg p-4 mb-3">
              <div className="text-sm text-muted-foreground mb-2">
                环 ID: <code className="text-xs">{cycle.id}</code>
              </div>
              <div className="space-y-1">
                {cycle.swaps.map((leg, i) => (
                  <div key={i} className="text-sm">
                    <code className="text-xs">{leg.from_user_id.slice(0, 8)}</code>
                    {" → "}
                    <code className="text-xs">{leg.to_user_id.slice(0, 8)}</code>
                    <span className="text-muted-foreground">
                      {" "}（提供 {leg.offer_item_id.slice(0, 8)}，想要{" "}
                      {leg.want_item_id.slice(0, 8)}）
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {error && <p className="mt-4 text-red-500 text-sm">{error}</p>}
    </div>
  );
}
