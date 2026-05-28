use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;

/// WebSocket 广播消息类型
#[derive(Debug, Clone, Serialize)]
pub struct WsNotification {
    pub event: String,
    pub data: serde_json::Value,
}

/// WebSocket hub，持有 broadcast sender 供其它模块发送通知
pub struct WsHub {
    tx: tokio::sync::broadcast::Sender<String>,
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

impl WsHub {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(256);
        Self { tx }
    }

    /// 向所有连接的客户端广播通知
    pub fn notify(&self, notification: &WsNotification) {
        if let Ok(json) = serde_json::to_string(notification) {
            // 没有活跃订阅者时忽略错误
            let _ = self.tx.send(json);
        }
    }

    /// 创建一个新的 broadcast 订阅者
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

/// WebSocket 升级处理：接受连接，转发广播消息到客户端
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<std::sync::Arc<crate::AppState>>,
) -> impl IntoResponse {
    let rx = state.ws_hub.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

/// 处理单个 WebSocket 连接的生命周期
async fn handle_socket(socket: WebSocket, mut rx: tokio::sync::broadcast::Receiver<String>) {
    let (mut sender, mut receiver) = socket.split();

    // 转发任务：将 broadcast 消息推送给客户端
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // 接收任务：读取客户端消息（目前只做心跳，防止连接被代理断开）
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Ping(data) = msg {
                tracing::debug!("收到 WebSocket ping: {:?}", data);
            }
        }
    });

    // 任一任务结束则清理连接
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::debug!("WebSocket 连接已关闭");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_hub_new() {
        let hub = WsHub::new();
        // 确保可以正常创建
        let _rx = hub.subscribe();
    }

    #[test]
    fn test_ws_hub_default() {
        let hub = WsHub::default();
        let _rx = hub.subscribe();
    }

    #[test]
    fn test_ws_hub_notify_no_subscribers() {
        let hub = WsHub::new();
        // 无订阅者时 notify 不应 panic
        hub.notify(&WsNotification {
            event: "test".into(),
            data: serde_json::json!({"key": "value"}),
        });
    }

    #[tokio::test]
    async fn test_ws_hub_broadcast() {
        let hub = WsHub::new();
        let mut rx = hub.subscribe();

        let notification = WsNotification {
            event: "match_found".into(),
            data: serde_json::json!({"item_id": "abc", "cycles_count": 1}),
        };
        hub.notify(&notification);

        let msg = rx.recv().await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["event"], "match_found");
        assert_eq!(parsed["data"]["cycles_count"], 1);
    }

    #[tokio::test]
    async fn test_ws_hub_multiple_subscribers() {
        let hub = WsHub::new();
        let mut rx1 = hub.subscribe();
        let mut rx2 = hub.subscribe();

        hub.notify(&WsNotification {
            event: "test".into(),
            data: serde_json::json!(null),
        });

        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }
}
