-- 003 性能索引 + 数据完整性约束

-- items 表：按属主和状态查询
CREATE INDEX IF NOT EXISTS idx_items_owner_id ON items(owner_id);
CREATE INDEX IF NOT EXISTS idx_items_status ON items(status);
CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at DESC);

-- items.tags GIN 索引：加速数组重叠查询（用于匹配引擎标签比对）
CREATE INDEX IF NOT EXISTS idx_items_tags ON items USING GIN(tags);

-- demands 表：按用户和关联物品查询
CREATE INDEX IF NOT EXISTS idx_demands_user_id ON demands(user_id);
CREATE INDEX IF NOT EXISTS idx_demands_offer_item_id ON demands(offer_item_id);
CREATE INDEX IF NOT EXISTS idx_demands_created_at ON demands(created_at DESC);

-- demands 标签 GIN 索引：加速匹配引擎的 offer_tags/target_tags 重叠查询
CREATE INDEX IF NOT EXISTS idx_demands_offer_tags ON demands USING GIN(offer_tags);
CREATE INDEX IF NOT EXISTS idx_demands_target_tags ON demands USING GIN(target_tags);

-- swap_cycles 表：按创建时间排序
CREATE INDEX IF NOT EXISTS idx_swap_cycles_created_at ON swap_cycles(created_at DESC);

-- 外键约束：demands.offer_item_id → items.id
ALTER TABLE demands
    ADD CONSTRAINT fk_demands_offer_item
    FOREIGN KEY (offer_item_id) REFERENCES items(id)
    ON DELETE CASCADE;

-- 外键约束：items.owner_id → users.id
ALTER TABLE items
    ADD CONSTRAINT fk_items_owner
    FOREIGN KEY (owner_id) REFERENCES users(id)
    ON DELETE CASCADE;

-- 外键约束：demands.user_id → users.id
ALTER TABLE demands
    ADD CONSTRAINT fk_demands_user
    FOREIGN KEY (user_id) REFERENCES users(id)
    ON DELETE CASCADE;
