# ELEVE 架构审查分析报告

> 日期：2026-08-13  
> 审查范围：24 个 crate，540 个 .rs 文件

---

## 一、依赖关系图

### 1.1 层级结构

```
层级 0（基础）：
  eleve-core → eleve-sysutil（唯一依赖）
  eleve-telemetry（无内部依赖）

层级 1（核心基础设施）：
  eleve-store → eleve-core
  eleve-lsp → eleve-core
  eleve-auth → eleve-core

层级 2（业务逻辑）：
  eleve-config → eleve-core, eleve-auth
  eleve-tools → eleve-core, eleve-auth, eleve-config
  eleve-cognitive → eleve-core, eleve-config
  eleve-model → eleve-core, eleve-auth, eleve-config

层级 3（高级功能）：
  eleve-routing → eleve-core, eleve-model, eleve-config, eleve-auth
  eleve-plugin → eleve-core, eleve-store, eleve-config

层级 4（上帝 crate）：
  eleve-tools-native → 9 个依赖
  eleve-agent → 11 个依赖
  eleve-app → 9 个依赖

层级 5（超级上帝 crate）：
  eleve-gateway → 12 个依赖

层级 6（入口）：
  eleve-bin → 9 个依赖
  eleve-cli → 8 个依赖
```

### 1.2 依赖统计

- **总 crate 数**：24 个
- **无内部依赖**：3 个（core, telemetry, cli）
- **依赖最多**：eleve-gateway（12 个）
- **被依赖最多**：eleve-core（23 个）
- **循环依赖**：无 ✅

### 1.3 架构评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 依赖方向 | ✅ 优秀 | 单向依赖，无循环 |
| 模块职责 | ⚠️ 一般 | 3 个上帝 crate（历史遗留） |
| 依赖解耦 | ⚠️ 一般 | gateway/tools-native/agent 依赖过重 |

---

## 二、eleve-core API 设计评估

### 2.1 优点

- Session 模块采用良好的 trait 分层设计：
  - `SessionStore`（同步，状态容器）
  - `SessionManager`（异步，生命周期管理）
  - `SessionQuery`（异步，只读查询）
  - `ProfileManager`（组合 trait，ISP 原则）
- 核心类型（Message, ToolCall, ContentPart）定义清晰
- 工具函数（safe_truncate 系列）有完整文档

### 2.2 问题

#### 🔴 lib.rs 过大（6436 行）

32 个 pub mod 全暴露，职责不清：

```rust
pub mod bootstrap;              // 启动逻辑，前端不需要
pub mod checkpoint_manager;     // 检查点管理，内部实现
pub mod error_classifier;       // 错误分类，内部实现
pub mod json_repair;            // JSON 修复，内部实现
pub mod reasoning_renderer;     // 推理渲染，内部实现
pub mod shells;                 // Shell 实现，内部实现
```

**建议**：用 `pub(crate)` 或重组为子模块。

#### 🔴 Message 结构体字段过多（18+ 个）

包含大量内部实现细节：

```rust
pub struct Message {
    // 内部脚手架标记，不应暴露
    pub empty_recovery_synthetic: Option<bool>,
    pub display_kind: Option<String>,
    pub api_content: Option<String>,
    
    // Provider 特定字段，应封装
    pub reasoning_details: Option<Vec<serde_json::Value>>,
    pub codex_reasoning_items: Option<Vec<serde_json::Value>>,
    pub codex_message_items: Option<Vec<serde_json::Value>>,
    
    // 平台特定字段
    pub platform_message_id: Option<String>,
}
```

**前端只需要**：role, content, tool_calls, timestamp, is_streaming

**建议**：拆分为核心字段 + 扩展字段，或提供 MessageView 适配层。

#### 🟡 async/sync 边界不统一

- `SessionStore` 是同步的
- `SessionManager` 是异步的
- 混用可能导致死锁

**建议**：统一为 async，或提供清晰的 async wrapper。

### 2.3 对 Iced 前端的影响

**可以直接接入**，但需要加一层适配：

```
eleve-desktop (Iced)
    ↓ 调用
eleve-ui-api (新建 facade)  ← 面向前端的简化类型
    ↓ 调用
eleve-core (现有)
```

---

## 三、重复代码分析

### 3.1 🔴 严重重复（需要立即修复）

#### 1. SessionState — 3 个独立定义

| 位置 | 用途 |
|------|------|
| `eleve-acp/src/session.rs:35` | ACP 协议的会话状态（运行时、含 Mutex） |
| `eleve-agent/src/session_state.rs:34` | Agent 可序列化状态（对标 Hermes AIAgent 属性） |
| `eleve-core/src/session/state.rs:17` | Core 层会话状态（session_id + messages + capabilities） |

**问题**：三个 crate 各自定义 SessionState，字段高度重叠，互不共享。

#### 2. Message — 2 个独立定义

| 位置 | 用途 |
|------|------|
| `eleve-core/src/lib.rs:594` | 核心消息类型（15+ 字段） |
| `eleve-store/src/session_db.rs:116` | 数据库消息记录（扁平化 DB 映射） |

**问题**：store 的 Message 是 core::Message 的 DB 映射，转换逻辑散落各处。

#### 3. PluginManager — 2 个同名 struct

| 位置 | 用途 |
|------|------|
| `eleve-tools/src/plugin_manager.rs:126` | WASM 插件管理 |
| `eleve-plugin/src/manager.rs:42` | 插件生命周期管理 |

**问题**：两个完全不同的 PluginManager，职责重叠但实现分离。

### 3.2 🟡 工具函数重复

#### 1. current_time() — 3 处完全相同

```rust
fn current_time() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64()).unwrap_or(0.0)
}
```

| 位置 |
|------|
| `eleve-agent/src/rate_limit.rs:80` |
| `eleve-agent/src/cross_session_rate_guard.rs:66` |
| `eleve-agent/src/credits_tracker.rs:538` |

#### 2. generate_uuid8() — 3 处重复

| 位置 |
|------|
| `eleve-agent/src/session_id.rs:15` |
| `eleve-core/src/session/mod.rs:42` |
| `eleve-store/src/utils.rs:28` |

#### 3. normalize_path() — 5 处重复

| 位置 |
|------|
| `eleve-core/src/path_utils.rs:12` |
| `eleve-tools/src/file_ops.rs:45` |
| `eleve-tools-native/src/fs.rs:78` |
| `eleve-agent/src/workspace.rs:33` |
| `eleve-store/src/session_db.rs:89` |

#### 4. extract_text_from_content() — 3 处重复

| 位置 |
|------|
| `eleve-core/src/message.rs:156` |
| `eleve-agent/src/conversation.rs:234` |
| `eleve-gateway/src/handlers.rs:89` |

#### 5. sanitize_filename() — 3 处重复

| 位置 |
|------|
| `eleve-tools/src/file_ops.rs:112` |
| `eleve-tools-native/src/fs.rs:156` |
| `eleve-agent/src/workspace.rs:78` |

#### 6. parse_duration_minutes() — 3 处重复

| 位置 |
|------|
| `eleve-config/src/parser.rs:45` |
| `eleve-agent/src/rate_limit.rs:123` |
| `eleve-cognitive/src/context.rs:67` |

### 3.3 🟡 命名模式重复

- 20+ 个 `*Store` trait/struct
- 20+ 个 `*Manager` trait/struct
- 13 个 `*Error` enum

**问题**：命名不统一，增加认知负担。

### 3.4 重复代码统计

| 类型 | 数量 | 严重度 |
|------|------|--------|
| 同名但不同实现的 struct | 3 组（SessionState ×3, Message ×2, PluginManager ×2） | 🔴 |
| 功能相似的 trait/struct | 20+ 个 *Store、20+ 个 *Manager、13 个 *Error | 🟡 |
| 可提取为公共 crate 的工具函数 | ~15 个函数 | 🟡 |

---

## 四、架构建议

### 4.1 短期（不影响现有开发）

1. **新建 eleve-ui-api crate**（或在 eleve-desktop 内部实现）
   - 定义面向前端的简化类型（MessageView, SessionSummary）
   - 提供响应式 API（Stream-based）
   - 隔离 tokio 依赖

2. **提取重复工具函数到 eleve-core**
   - `current_time()` → `eleve_core::time::current_time()`
   - `generate_uuid8()` → `eleve_core::id::generate_uuid8()`
   - `normalize_path()` → `eleve_core::path::normalize_path()`
   - 其他 12 个函数

### 4.2 中期（需要重构）

1. **统一 SessionState**
   - 以 `eleve-core::session::SessionState` 为准
   - 其他 crate 使用 core 的版本，或定义自己的适配层

2. **统一 Message 类型**
   - core 的 Message 保持不变
   - store 的 Message 改为 core::Message 的 DB 映射，提供 `From` trait 转换

3. **重命名 PluginManager**
   - `eleve-tools::PluginManager` → `WasmPluginManager`
   - `eleve-plugin::PluginManager` → `PluginLifecycleManager`

### 4.3 长期（架构优化）

1. **引入 trait 层**
   - 创建 `eleve-traits` crate，定义核心接口
   - 减少具体实现间的直接依赖

2. **依赖注入**
   - 对于 telemetry、plugin 等可选功能，使用依赖注入而非硬依赖

3. **拆分上帝 crate**
   - 将 gateway 拆分为 `gateway-core`（接口）和 `gateway-impl`（实现）
   - 重新评估 eleve-tools-native，考虑将其功能分散到各个专门的 crate

---

## 五、结论

### 5.1 架构健康度

| 维度 | 评分 | 说明 |
|------|------|------|
| 依赖方向 | ✅ 优秀 | 单向依赖，无循环 |
| 模块职责 | ⚠️ 一般 | 3 个上帝 crate（历史遗留） |
| API 设计 | ⚠️ 一般 | core 的 lib.rs 过大，Message 字段过多 |
| 重复代码 | 🔴 严重 | 3 组同名 struct，15+ 个重复函数 |
| Rust 反模式 | ⚠️ 轻微 | 上帝 crate |

### 5.2 对 Iced 前端的影响

- **可以直接接入**，但需要加适配层
- **重复代码不影响**，eleve-desktop 独立仓库，不依赖这些重复
- **上帝 crate 不影响**，eleve-desktop 只依赖 core/agent/config/model

### 5.3 优先级

1. **P0**：创建 eleve-ui-api（或在 eleve-desktop 内部实现）
2. **P1**：提取重复工具函数到 eleve-core
3. **P2**：统一 SessionState、Message 类型
4. **P3**：重命名 PluginManager，拆分上帝 crate

---

**附录：参考文件**

- 依赖关系详细分析：`subagent-summary-0-20260813_185011_370072.txt`
- API 设计详细分析：`subagent-summary-1-20260813_185011_370072.txt`
- 重复代码详细分析：`subagent-summary-2-20260813_185011_371003.txt`
