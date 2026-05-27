-- 木桃 Mutao 数据库初始化
CREATE TABLE IF NOT EXISTS items (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    image_url TEXT NOT NULL DEFAULT '',
    tags TEXT[] NOT NULL DEFAULT '{}',
    value_tier SMALLINT NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'Idle',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS demands (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    offer_item_id UUID NOT NULL,
    offer_tags TEXT[] NOT NULL DEFAULT '{}',
    target_tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS swap_cycles (
    id UUID PRIMARY KEY,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
