"use client";

import { useEffect, useState } from "react";
import { api, type Demand } from "@/lib/api";

export default function DemandsPage() {
  const [demands, setDemands] = useState<Demand[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    api
      .listDemands()
      .then(setDemands)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="container mx-auto p-8">加载中...</div>;
  if (error)
    return <div className="container mx-auto p-8 text-red-500">{error}</div>;

  return (
    <div className="container mx-auto p-8">
      <h1 className="text-2xl font-bold mb-6">交换意向</h1>

      {demands.length === 0 ? (
        <p className="text-muted-foreground">暂无交换意向</p>
      ) : (
        <div className="space-y-3">
          {demands.map((d) => (
            <div key={d.id} className="border rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-muted-foreground">
                  用户: <code className="text-xs">{d.user_id.slice(0, 8)}</code>
                </span>
                <span className="text-xs text-muted-foreground">
                  {new Date(d.created_at).toLocaleString("zh-CN")}
                </span>
              </div>
              <div className="flex items-center gap-4">
                <div>
                  <span className="text-xs text-muted-foreground">提供:</span>
                  <div className="flex gap-1 mt-1">
                    {d.offer_tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
                <span className="text-muted-foreground">→</span>
                <div>
                  <span className="text-xs text-muted-foreground">想要:</span>
                  <div className="flex gap-1 mt-1">
                    {d.target_tags.map((tag) => (
                      <span
                        key={tag}
                        className="text-xs px-2 py-0.5 rounded-full bg-blue-100 text-blue-700"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
