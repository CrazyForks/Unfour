# API 默认集合替代未分组方案

## 背景与目标

当前 API Client 使用 `collection_id IS NULL` 表示"未分组"请求，这导致：
- 数据模型有特殊状态，每处涉及集合的逻辑都要考虑 NULL 情况
- UI 有大量特殊分支（未分组不能重命名、不能删除、没有右键菜单等）
- 与 Postman 等主流工具的心智模型不一致

**目标**：用"默认集合"替代"未分组"概念，当 workspace 下没有任何集合时，保存请求自动创建一个默认集合。

## 核心设计原则

1. **数据模型统一**：所有新保存的请求都有 `collection_id`，不再有 NULL 状态
2. **惰性创建**：只有当 workspace 下没有集合且首次保存请求时，才自动创建默认集合
3. **用户无感**：对用户来说，体验变化不大（第一个集合始终在那里），但代码更干净
4. **不处理历史数据**：已有 `collection_id IS NULL` 的请求保持原样，不做迁移

## 改动范围

### 后端（Rust）

#### 1. `crates/http-engine/src/api_client.rs`
- `save_request()`: 当 `collection_id` 为 None 且 workspace 无集合时，自动创建默认集合并使用其 ID
- `saved_request_fields()`: 调整 `collection_id` 的处理逻辑，不再允许 None
- `list_collections()`: 移除构造"General"假集合的逻辑（未分组不再存在）
- `move_request()`: `collection_id` 参数改为必选（或 None 时使用默认集合）
- `normalize_collection_id()`: 保留但语义变化（空字符串 → 使用默认集合）

#### 2. `crates/unfour-command-bus/src/lib.rs`
- `ApiListCollections` 处理：移除未分组计数和假集合构造
- `ApiSaveRequest` / `ApiUpdateRequest`：确保 collection_id 最终不为空

#### 3. `crates/unfour-mcp/src/tools/api.rs`
- MCP 工具中 `collection_id` 字段处理：从 `unwrap_or_default()` 改为确保有值
- 测试数据更新：移除 "General" 假集合

### 前端（TypeScript）

#### 4. `packages/api-client/src/request-utils.ts`
- `groupRequestsByCollection()`: 移除未分组分支，直接按真实 collectionId 分组
- 移除对 `collectionId ?? null` 的特殊处理

#### 5. `packages/api-client/src/components/ApiCollectionTree.tsx`
- 移除未分组相关的特殊渲染逻辑
- `folderToTreeItem()`: 移除 `collectionId ?? "unfiled"` 的 fallback
- `requestTreeItem()`: 移除"移到未分组"菜单项
- 集合列表就是纯粹的集合，没有特殊项

#### 6. `packages/api-client/src/components/ApiSaveDialog.tsx`
- 移除 `UNFILED_KEY` 常量和未分组选项
- 默认选中第一个集合（或最近使用的集合）
- 当没有集合时，提示用户创建集合（或由后端自动创建，前端无感）

#### 7. `packages/api-client/src/model/request-tabs.ts`
- `RequestDraft.collectionId`: 考虑是否改为必选（string 而非 string | null）
- `emptyDraft()`: collectionId 默认值调整

#### 8. `packages/api-client/src/hooks/useApiRequestTabs.ts`
- `saveTab()`: 确保保存时 collectionId 不为空
- `tabToInput()`: collectionId 处理调整

### i18n

#### 9. `packages/ui/src/i18n/locales/en.json`
#### 10. `packages/ui/src/i18n/locales/zh-CN.json`
- 移除 `api.collection.unfiled` 等未分组相关的 key
- 新增默认集合名称相关 key（如 `api.collection.defaultName`）

### 测试文件

#### 11. `packages/api-client/src/request-utils.test.ts`
- 更新分组测试用例，移除未分组相关测试

#### 12. `packages/api-client/src/components/ApiCollectionTree.test.tsx` (如有)
- 更新测试用例

#### 13. `crates/http-engine/src/api_client.rs` 中的测试
- 更新保存请求、集合列表的测试用例
- 添加默认集合自动创建的测试

## 实施步骤

### 阶段一：后端改造
1. 在 `api_client.rs` 中添加 `ensure_default_collection()` 方法
2. 修改 `save_request()`，当 collection_id 为 None 时调用 ensure_default_collection
3. 修改 `list_collections()`，移除未分组假集合
4. 修改 `move_request()` 注释和行为
5. 更新后端测试

### 阶段二：前端改造
1. 修改 `request-utils.ts` 中的 `groupRequestsByCollection()`
2. 修改 `ApiCollectionTree.tsx`，移除未分组特殊逻辑
3. 修改 `ApiSaveDialog.tsx`，移除 UNFILED_KEY
4. 更新 i18n keys
5. 更新前端测试

### 阶段三：集成验证
1. 运行完整测试套件
2. 手动验证：新建 workspace → 保存请求 → 自动创建默认集合
3. 手动验证：删除最后一个集合后再保存请求的行为

## 默认集合的命名

- 英文："My Collection" 或 "Default"
- 中文："默认集合" 或 "我的集合"

建议使用 "My Collection" / "默认集合"，与 Postman 对齐。

## 风险与应对

### 风险 1：用户删除了所有集合后再保存请求
**应对**：
- 保存请求时，如果 workspace 没有任何集合，自动创建默认集合
- 这个逻辑在 `save_request()` 中处理，前端无感

### 风险 2：MCP 工具返回值变化
**应对**：
- 确保 `collection_id` 字段始终有值（新请求）
- 历史数据中仍可能为空字符串，MCP 消费者需要兼容

### 风险 3：前端类型变化（collectionId 从可选变必选）
**应对**：
- 保持类型为 `string | null`，新数据运行时确保不为 null
- 历史数据可能仍为 null，UI 层做兼容处理

## 不需要改动的部分

- 数据库表结构不需要改变（`collection_id` 保持可空，只是业务逻辑上确保不为空）
- API 请求/响应的 schema 不需要破坏性变更
- SSH、Database、Workspace 等其他模块完全不受影响

## 验证清单

- [ ] 新建 workspace，保存请求，自动创建默认集合
- [ ] 删除所有集合后保存请求，重新创建默认集合
- [ ] 集合树中没有未分组特殊项（新数据）
- [ ] 保存对话框中没有未分组选项
- [ ] 请求右键菜单中没有"移到未分组"选项
- [ ] MCP `list_collections` 返回真实集合列表（无假集合）
- [ ] 新保存的请求 `collectionId` 有值
- [ ] 所有现有测试通过
- [ ] 新功能有对应测试
