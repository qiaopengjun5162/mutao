"use client";

import { useEffect, useState, useCallback } from "react";

export interface WsNotification {
  event: string;
  data: Record<string, unknown>;
  time: Date;
}

// WebSocket 实时通知 hook，组件挂载时自动连接，卸载时断开
export function useWs() {
  const [notifications, setNotifications] = useState<WsNotification[]>([]);
  const [connected, setConnected] = useState(false);

  const addNotification = useCallback((event: string, data: unknown) => {
    setNotifications((prev) => [
      { event, data: data as Record<string, unknown>, time: new Date() },
      ...prev.slice(0, 49),
    ]);
  }, []);

  useEffect(() => {
    const wsBase = (process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000").replace(/^http/, "ws");
    const ws = new WebSocket(`${wsBase}/api/ws`);
    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data);
        addNotification(msg.event, msg.data);
      } catch { /* 忽略非 JSON 消息 */ }
    };
    return () => ws.close();
  }, [addNotification]);

  return { notifications, connected };
}
