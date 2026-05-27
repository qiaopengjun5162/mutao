const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000";

// Auth types
export interface LoginRes {
  token: string;
  user_id: string;
  username: string;
}

export interface RegisterReq {
  username: string;
  password: string;
}

export interface LoginReq {
  username: string;
  password: string;
}

export interface Item {
  id: string;
  owner_id: string;
  title: string;
  description: string;
  image_url: string;
  tags: string[];
  value_tier: number;
  status: string;
  created_at: string;
}

export interface Demand {
  id: string;
  user_id: string;
  offer_item_id: string;
  offer_tags: string[];
  target_tags: string[];
  created_at: string;
}

export interface SwapLeg {
  from_user_id: string;
  to_user_id: string;
  offer_item_id: string;
  want_item_id: string;
}

export interface SwapCycle {
  id: string;
  swaps: SwapLeg[];
  created_at: string;
}

export interface CreateItemReq {
  owner_id: string;
  title: string;
  description?: string;
  image_url?: string;
  tags: string[];
  value_tier: number;
}

export interface CreateDemandReq {
  user_id: string;
  offer_item_id: string;
  offer_tags: string[];
  target_tags: string[];
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `请求失败: ${res.status}`);
  }
  return res.json();
}

export const api = {
  // Auth
  register: (data: RegisterReq) =>
    request<LoginRes>("/api/auth/register", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  login: (data: LoginReq) =>
    request<LoginRes>("/api/auth/login", {
      method: "POST",
      body: JSON.stringify(data),
    }),

  // Items
  listItems: () => request<Item[]>("/api/items"),
  getItem: (id: string) => request<Item>(`/api/items/${id}`),
  createItem: (data: CreateItemReq) =>
    request<Item>("/api/items", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  matchItem: (id: string) =>
    request<SwapCycle[]>(`/api/items/${id}/match`, { method: "POST" }),
  updateItemStatus: (id: string, status: string) =>
    request<Item>(`/api/items/${id}/status`, {
      method: "PATCH",
      body: JSON.stringify({ status }),
    }),
  listDemands: () => request<Demand[]>("/api/demands"),
  createDemand: (data: CreateDemandReq) =>
    request<Demand>("/api/demands", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  listCycles: () => request<SwapCycle[]>("/api/cycles"),
  confirmSwap: (id: string) =>
    request<{ cycle_id: string; status: string; items_completed: number }>(
      `/api/cycles/${id}/confirm`,
      { method: "POST" }
    ),
  health: () => request<{ status: string; name: string }>("/api/health"),
};
