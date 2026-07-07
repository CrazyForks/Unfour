# 删除与墓碑回收（GC）策略设计

> 状态：提案（仅评审，未改代码）
> 背景：数据库后期要支持云同步，所有业务表已采用软删除（`deleted_at` + `revision` + `sync_status` + `remote_id`）。
> 目标：写入路径统一软删除不变；在**回收（物理删除）**阶段按工作区同步域采用不同策略，兼顾「同步正确性」与「本地库精简」。

---

## 1. 目标与原则

1. **写入统一软删除**：所有删除只置 `deleted_at`、 `revision+1`、 `sync_status='deleted'`。单一写入路径，天然同步安全。
2. **回收按域分化**：后台 GC 作业对「本地域」与「同步域」应用不同资格（宽限期 + 云确认闸门），但用同一个作业、同一套调度。
3. **云是真相源**：同步域的墓碑在被云端确认前**绝不被物理删除**，避免云端留孤儿、设备间状态分叉。
4. **本地域即时精简**：无云端可通知，墓碑只是死重，按短宽限期每日回收。

---

## 2. 同步域（sync scope）定义

判定以 **workspace** 为单位（记录通过 `workspace_id` 归属工作区）：

| 域 | 判定条件 | 含义 |
|----|----------|------|
| 本地域 | `remote_id IS NULL` 且 `sync_status = 'local'`（且当前无已登录账户/未启用云） | 该工作区永不同步 |
| 同步域 | `remote_id IS NOT NULL`（或等效的「已启用云同步」标志） | 该工作区有云端归属 |

- GC **每次运行都重新读取** workspace 的同步状态，不长期缓存（工作区同步域会变：本地 → 启用同步）。
- 后期引入用户/账户机制后，「同步域」定义改为「工作区归属于已登录且启用云的用户」——**机制形状相同**（仍是一个标记工作区云端归属的列/标志），GC 逻辑无需重写。

---

## 3. `sync_status` 状态机

当前可见值：`local`、 `deleted`。扩展如下：

```
               创建/编辑
  ┌──────────────────────────────┐
  │                              ▼
[local] ──启用同步──▶ [synced] ──删除──▶ [deleted] ──同步引擎推送并云端确认──▶ [deleted_synced] ──GC──▶ (物理删除)
                              ▲                                                        │
                              └──────────────── 撤销(宽限期内) ◀──────────────────────┘
```

- `local`：本地创建，尚未推送。
- `synced`：已与云端一致（正常态；初版若未细分可暂用单值，本方案不强制）。
- `deleted`：已软删，**墓碑待推送**至云端。
- `deleted_synced`：墓碑已成功推送并被云端确认 → **安全可被 GC 回收**。

**转换触发**：
- 删除：`local/synced` → `deleted`（写路径，已存在）。
- 同步引擎：推送删除成功后 → `deleted` → `deleted_synced`（需在同步引擎中加一步状态翻转）。
- 撤销（宽限期内）：`deleted` / `deleted_synced` → 清 `deleted_at` 回 `synced`（仅本地域或同步域宽限期内）。

**备选（时间戳方案）**：不新增 `deleted_synced` 状态，而是给 `workspaces` 增加 `last_sync_success_at`；GC 资格用 `deleted_at < workspaces.last_sync_success_at`（删除发生在上次成功同步之前 ⇒ 必已被推送）。状态更少，但依赖同步引擎可靠写入该时间戳。本方案以**状态机为主推**（语义最清晰）。

> 注：初版迁移中 `workspaces` / `connections` 的 `sync_status` 未见 `CHECK` 约束；若存在则扩展其取值包含 `deleted_synced`，否则跳过。

---

## 4. 迁移草案（0003，仅示意）

```sql
-- 4.1 扩展 sync_status 取值（仅当现有有 CHECK 时；当前未见，按需）
-- ALTER TABLE connections ADD CONSTRAINT ... 视实际 CHECK 调整

-- 4.2 为 GC 查询建复合索引（覆盖 sync_status + deleted_at + workspace_id）
CREATE INDEX IF NOT EXISTS idx_connections_gc
  ON connections(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_gc
  ON workspaces(sync_status, deleted_at, remote_id);
CREATE INDEX IF NOT EXISTS idx_api_collections_gc
  ON api_collections(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_collection_folders_gc
  ON api_collection_folders(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_requests_gc
  ON api_requests(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_history_gc
  ON api_history(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_environments_gc
  ON api_environments(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_saved_sql_gc
  ON saved_sql(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_db_query_history_gc
  ON db_query_history(sync_status, deleted_at, workspace_id);
CREATE INDEX IF NOT EXISTS idx_activity_events_gc
  ON activity_events(sync_status, deleted_at, workspace_id);

-- 4.3 （备选时间戳方案）workspaces 增加上次成功同步时间
-- ALTER TABLE workspaces ADD COLUMN last_sync_success_at TEXT;
```

**GC 作用范围说明**：
- `ssh_connections` / `database_connections` **无** `deleted_at`，其删除经由 `connections` 物理删除时的 `ON DELETE CASCADE` 自动清理 → GC **只操作顶层含 `deleted_at` 的表**，子表靠级联。
- `workspaces` 自身的墓碑较特殊：本地域可在宽限后物理删除；同步域建议保留至云端确认移除（与评审 #6 的级联策略一致）。
- `api_requests` 等子表若对父表（如 `api_collections`）未设 `ON DELETE CASCADE`，GC 删父表墓碑时子表墓碑会残留 → 需确认并补全级联（呼应评审 #6）。

---

## 5. GC 作业伪代码

```text
fn run_gc(pool, now):
    local_grace  = days(7)    # 本地域宽限（无撤销需求可降到 1d）
    synced_grace = days(30)   # 同步域宽限（跨设备撤销窗口）
    batch        = 500

    # 预先分别取出两类 workspace 的 id 集合（每次运行重算，不缓存）
    local_ws  = SELECT id FROM workspaces WHERE remote_id IS NULL AND sync_status = 'local'
    synced_ws = SELECT id FROM workspaces WHERE remote_id IS NOT NULL

    for T in [connections, api_collections, api_collection_folders,
              api_requests, api_history, api_environments,
              saved_sql, db_query_history, activity_events, workspaces]:
        loop:
            rows = SELECT id FROM T
                   WHERE deleted_at IS NOT NULL
                     AND (
                       (workspace_id IN local_ws
                          AND deleted_at < now - local_grace)
                       OR
                       (workspace_id IN synced_ws
                          AND sync_status = 'deleted_synced'
                          AND deleted_at < now - synced_grace)
                     )
                   LIMIT batch
            if rows empty: break
            DELETE FROM T WHERE id IN (rows)   # 事务内、按批提交，级联清子表
```

- **本地域**：不看云确认，短宽限后直接回收。
- **同步域**：必须 `deleted_synced`（云已确认）且过宽限才回收。
- 每批一个事务，提交后可中断续跑；索引保证扫描廉价。

---

## 6. 调度与运维

- **频率**：每日一次，或在 app 空闲 / 启动且距上次 GC > 24h 时触发。两者用同一作业，区别仅在资格。
- **记账**：`app_settings` 记录 `last_gc_at`，避免重复/重叠运行。
- **资源**：低优先级后台任务，分批 + 事务，避免长时间锁本地库。
- **监控**：记录每次回收行数，便于观测墓碑积累速率。

---

## 7. 边界与风险

1. **云确认闸门（最关键）**：同步域 GC 绝不能删「同步引擎还没推过」的墓碑，否则云端永远学不到删除 → 设备间分叉。靠 `deleted_synced` 状态或 `last_sync_success_at` 保证。
2. **工作区同步域会变**：本地工作区启用同步后，已被 GC 清掉的删除＝云从未有（无所谓）；尚未回收的墓碑自动转为同步域规则。
3. **换机 / 全量重同步**：云是真相源，本地 GC 只清「已确认同步」墓碑，新设备从云拉取仍能拿到删除指令。
4. **撤销 UX**：若 app 内提供「最近删除」回收站，宽限期须覆盖该窗口（每日 GC + 7d 本地宽限 ⇒ 7 天可回收站）。
5. **FK 级联一致性（评审 #6）**：GC 是物理 DELETE，须确认父→子墓碑关系的 `ON DELETE CASCADE`，否则留孤儿行。
6. **查询过滤**：业务查询仍需 `WHERE deleted_at IS NULL`（或建视图/DAO 统一收敛），否则墓碑污染列表——软删方案的固有成本。

---

## 8. 迁移成本评估：现在做 vs 后期随用户/账户机制做

**结论先行**：后期再随用户机制加入，\*\*迁移本身的复杂度并不显著更大\*\*；真正随延期增长的是「墓碑数据体积」（运营负担），而非迁移难度。

### 现在做（在用户机制之前）
- 基础已具备：所有表已有 `deleted_at` / `sync_status` / `remote_id`，写路径（软删）已实现。
- 增量很小：一份迁移（扩 `sync_status` 取值 + 加 GC 索引）+ 一个 GC 作业 + 同步引擎翻转 `deleted→deleted_synced`。
- 障碍：同步域的「云确认闸门」依赖**同步引擎存在**；若同步引擎尚未开发，可先只上线**本地域每日 GC**（不依赖同步引擎），同步域闸门等同步引擎落地再补。

### 后期做（随用户/账户 + 同步引擎一起）
- ** schema 无需重写**：软删基础设施已就位且会被复用；后期主要加「GC 作业 + 把域判定接到账户」，不是重型迁移。
- **真正的重型工作是同步引擎本身**（与 GC 正交，迟早要做），不是 GC 的迁移。
- **延期代价是运营性的**：墓碑无限堆积，库膨胀、备份/同步载荷变大；但加索引后 GC 处理大表只是一次性 O(n)，不痛。
- **无数据回填压力**：前期所有行都是 `local`/软删，后期 GC 按「本地域」规则即可回收，不与用户隔离冲突（前期数据本就无用户归属）。

### 推荐落地路径（增量、每步可独立发布）
1. **现在**：实现「本地域每日 GC」（短宽限，无云依赖）。立即止住非同步数据的无限膨胀，且不依赖用户机制。
2. **用户/账户 + 同步引擎落地时**：扩展 GC，加入「同步域 + `deleted_synced` 云确认闸门」与按账户判定域。
3. 每步独立、可回退；无 big-bang 迁移。

---

## 9. 待确认项（决策前）

- `LOCAL_GRACE` / `SYNCED_GRACE` 具体天数（影响撤销窗口与存储）。
- 同步域判定是否仅用 `remote_id IS NOT NULL`，还是另加显式「云启用」标志。
- 采用状态机（`deleted_synced`）还是时间戳（`last_sync_success_at`）方案。
- `workspaces` 自身墓碑是否纳入物理 GC（本地域可回收，同步域建议保留至云端确认）。
- 是否需要在 app 内提供「最近删除」回收站（决定宽限期下限）。
