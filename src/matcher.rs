use crate::models::{Demand, SwapCycle, SwapLeg};
use std::collections::HashSet;
use uuid::Uuid;

pub struct Matcher;

impl Matcher {
    /// 在需求图中寻找所有长度在 [2, max_cycle_len] 的有效交换环
    ///
    /// 算法：对每个起始节点做 DFS 探索，当回到起始节点时记录为一个环
    /// 通过邻接表加速：i 的物品能满足 j 的需求时，i→j 有边
    pub fn find_cycles(demands: &[Demand], max_cycle_len: usize) -> Vec<SwapCycle> {
        if demands.len() < 2 {
            return vec![];
        }

        let mut adjacency: Vec<Vec<usize>> = vec![vec![]; demands.len()];
        for i in 0..demands.len() {
            for j in 0..demands.len() {
                if i == j {
                    continue;
                }
                // i 的物品（offer_tags）能满足 j 的需求（target_tags）时，i→j 有边
                if Self::tags_overlap(&demands[i].offer_tags, &demands[j].target_tags) {
                    adjacency[i].push(j);
                }
            }
        }

        let mut cycles = Vec::new();
        let mut visited = vec![false; demands.len()];
        let mut path = Vec::new();

        for start in 0..demands.len() {
            Self::dfs(
                start,
                start,
                &adjacency,
                demands,
                &mut visited,
                &mut path,
                &mut cycles,
                max_cycle_len,
            );
        }

        cycles
    }

    fn dfs(
        start: usize,
        current: usize,
        adj: &[Vec<usize>],
        demands: &[Demand],
        visited: &mut [bool],
        path: &mut Vec<usize>,
        cycles: &mut Vec<SwapCycle>,
        max_len: usize,
    ) {
        if path.len() >= max_len {
            return;
        }

        visited[current] = true;
        path.push(current);

        for &next in &adj[current] {
            if next == start && path.len() >= 2 {
                let cycle = Self::build_cycle(path, demands);
                if let Some(c) = cycle {
                    if !Self::exists(cycles, &c) {
                        cycles.push(c);
                    }
                }
            } else if !visited[next] {
                Self::dfs(start, next, adj, demands, visited, path, cycles, max_len);
            }
        }

        path.pop();
        visited[current] = false;
    }

    fn build_cycle(path: &[usize], demands: &[Demand]) -> Option<SwapCycle> {
        let n = path.len();
        if n < 2 {
            return None;
        }
        // 确保同一用户不出现多次（避免自己换自己）
        let mut users = HashSet::new();
        for &idx in path {
            if !users.insert(demands[idx].user_id) {
                return None; // 同一用户出现多次，无效环
            }
        }

        let mut swaps = Vec::new();
        for i in 0..n {
            let from = &demands[path[i]];
            // from 接收的是环中前一个人的物品（path[(i-1+n)%n]）
            let want = &demands[path[(i + n - 1) % n]];
            let to = &demands[path[(i + 1) % n]];
            swaps.push(SwapLeg {
                from_user_id: from.user_id,
                to_user_id: to.user_id,
                offer_item_id: from.offer_item_id,
                want_item_id: want.offer_item_id,
            });
        }
        let cycle = SwapCycle {
            id: Uuid::new_v4(),
            swaps,
            created_at: chrono::Utc::now(),
        };
        if cycle.is_valid() { Some(cycle) } else { None }
    }

    fn tags_overlap(a: &[String], b: &[String]) -> bool {
        let set: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
        b.iter().any(|s| set.contains(s.as_str()))
    }

    /// 检查候选环是否已存在于结果集中（排序后比较，忽略顺序差异）
    fn exists(cycles: &[SwapCycle], candidate: &SwapCycle) -> bool {
        let mut cand: Vec<(Uuid, Uuid)> = candidate
            .swaps
            .iter()
            .map(|leg| (leg.offer_item_id, leg.want_item_id))
            .collect();
        cand.sort();
        cycles.iter().any(|c| {
            let mut exist: Vec<(Uuid, Uuid)> =
                c.swaps.iter().map(|leg| (leg.offer_item_id, leg.want_item_id)).collect();
            exist.sort();
            cand == exist
        })
    }
}
