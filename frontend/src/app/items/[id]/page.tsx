"use client";

import { useEffect, useState, use } from "react";
import { api, type Item, type SwapCycle } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { useWs } from "@/hooks/use-ws";

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
  const [analyzing, setAnalyzing] = useState(false);
  const [attesting, setAttesting] = useState(false);
  const [error, setError] = useState("");
  const [aiResult, setAiResult] = useState<{ tags: string[]; value_tier: number; method: string } | null>(null);
  const [attestResult, setAttestResult] = useState<{ tx_hash: string; chain: string } | null>(null);
  const { notifications } = useWs();

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

  // AI 标签提取：调用 scalpel.py 后端
  const handleAnalyze = async () => {
    if (!item) return;
    setAnalyzing(true);
    setError("");
    try {
      const result = await api.analyzeItem(item.title + " " + (item.description || ""));
      setAiResult(result);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "AI 分析失败");
    } finally {
      setAnalyzing(false);
    }
  };

  // Web3 存证：物品流转履历上链
  const handleAttest = async () => {
    if (!item) return;
    setAttesting(true);
    setError("");
    try {
      const result = await api.attestItem(item.id);
      setAttestResult(result);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "存证失败");
    } finally {
      setAttesting(false);
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

        {/* 操作按钮组 */}
        <div className="flex flex-col sm:flex-row gap-2">
          {item.status === "Idle" && (
            <Button onClick={handleMatch} disabled={matching} className="flex-1">
              {matching ? "匹配中..." : "触发匹配"}
            </Button>
          )}
          <Button onClick={handleAnalyze} disabled={analyzing} variant="outline" className="flex-1">
            {analyzing ? "AI 分析中..." : "AI 标签提取"}
          </Button>
          <Button onClick={handleAttest} disabled={attesting} variant="outline" className="flex-1">
            {attesting ? "存证中..." : "Web3 存证"}
          </Button>
        </div>

        {/* AI 分析结果 */}
        {aiResult && (
          <div className="mt-4 p-3 rounded-md bg-secondary/50 text-sm">
            <p className="font-medium mb-1">AI 分析结果（{aiResult.method}）</p>
            <p>标签：{aiResult.tags.join(", ")}</p>
            <p>价值等级：{aiResult.value_tier}/5</p>
          </div>
        )}

        {/* Web3 存证结果 */}
        {attestResult && (
          <div className="mt-4 p-3 rounded-md bg-secondary/50 text-sm">
            <p className="font-medium mb-1">存证成功</p>
            <p>链：{attestResult.chain}</p>
            <p>交易哈希：<code className="text-xs break-all">{attestResult.tx_hash}</code></p>
          </div>
        )}

        {/* 实时通知 */}
        {notifications.length > 0 && (
          <div className="mt-4 space-y-1">
            <p className="text-xs font-medium text-muted-foreground">实时通知</p>
            {notifications.slice(0, 5).map((n, i) => (
              <div key={i} className="text-xs text-muted-foreground">
                <span className="font-mono">{n.event}</span>: {JSON.stringify(n.data)}
              </div>
            ))}
          </div>
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
