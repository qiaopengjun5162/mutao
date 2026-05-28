"use client";

import { useEffect, useState } from "react";
import { api, type SwapCycle } from "@/lib/api";
import { Button } from "@/components/ui/button";

export default function CyclesPage() {
  const [cycles, setCycles] = useState<SwapCycle[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [confirming, setConfirming] = useState<string | null>(null);

  useEffect(() => {
    api
      .listCycles()
      .then(setCycles)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  const handleConfirm = async (id: string) => {
    setConfirming(id);
    try {
      await api.confirmSwap(id);
      setCycles((prev) => prev.filter((c) => c.id !== id));
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "确认失败");
    } finally {
      setConfirming(null);
    }
  };

  if (loading) return <div className="container mx-auto p-4 md:p-8">加载中...</div>;
  if (error)
    return <div className="container mx-auto p-4 md:p-8 text-red-500">{error}</div>;

  return (
    <div className="container mx-auto p-4 md:p-8">
      <h1 className="text-xl md:text-2xl font-bold mb-6">交换环</h1>

      {cycles.length === 0 ? (
        <p className="text-muted-foreground">暂无交换环</p>
      ) : (
        <div className="space-y-4">
          {cycles.map((cycle) => (
            <div key={cycle.id} className="border rounded-lg p-4">
              <div className="flex items-center justify-between mb-3">
                <span className="text-sm font-medium">
                  {cycle.swaps.length} 人交换环
                </span>
                <code className="text-xs text-muted-foreground">
                  {cycle.id.slice(0, 8)}
                </code>
              </div>

              <div className="space-y-2 mb-4">
                {cycle.swaps.map((leg, i) => (
                  <div key={i} className="flex items-center gap-2 text-sm">
                    <code className="text-xs bg-secondary px-2 py-0.5 rounded">
                      {leg.from_user_id.slice(0, 8)}
                    </code>
                    <span className="text-muted-foreground">给出</span>
                    <code className="text-xs">{leg.offer_item_id.slice(0, 8)}</code>
                    <span className="text-muted-foreground">→</span>
                    <code className="text-xs bg-secondary px-2 py-0.5 rounded">
                      {leg.to_user_id.slice(0, 8)}
                    </code>
                    <span className="text-muted-foreground">得到</span>
                    <code className="text-xs">{leg.want_item_id.slice(0, 8)}</code>
                  </div>
                ))}
              </div>

              <Button
                onClick={() => handleConfirm(cycle.id)}
                disabled={confirming === cycle.id}
                className="w-full"
              >
                {confirming === cycle.id ? "确认中..." : "确认交换"}
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
