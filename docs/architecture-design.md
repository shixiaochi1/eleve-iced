# ELEVE Iced 架构设计方案

> 版本：0.3.0 (Draft)  
> 日期：2026-08-13  
> 状态：已对照 eleve-iced 实际实现对齐（2026-08-17）：iced 升级至 0.14；ui/ 改为按功能划分；events.rs 推迟至接入后端阶段

---

## 1. 项目背景

### 1.1 现状问题

当前 ELEVE 桌面端采用 Tauri + WebView 架构，存在以下问题：

- **内存占用高**：400-500MB（WebView 进程 + React 虚拟 DOM）
- **性能瓶颈**：JS 主线程阻塞、DOM 操作帧率下降、IPC 序列化开销
- **架构复杂**：双进程模型，需要 WS/HTTP 通信

### 1.2 目标

- **单进程架构**：直接链接 eleve-app，零 IPC 开销
- **内存优化**：目标 < 200MB
- **独立迭代**：不影响现有 Tauri 版本，可并行开发
- **原生体验**：GPU 加速渲染，60fps 流畅交互

---

## 2. 架构设计

### 2.1 核心原则

**eleve-iced 只是一个 UI 客户端**，和 eleve-gateway 平级，都是 eleve-app 的消费者。

```
现有架构：
  eleve-gateway (HTTP/WS) → eleve-app (Service) → eleve-core
  eleve-cli (CLI)         → eleve-app (Service) → eleve-core

eleve-iced 架构：
  eleve-iced (UI)         → eleve-app (Service) → eleve-core
```

### 2.2 极简架构

eleve-iced 不需要任何中间层，直接调用 eleve-app 的 Service：

```
┌─────────────────────────────────────┐
│  eleve-iced (UI 客户端)             │
│  - Iced 渲染                         │
│  - 调用 eleve-app::SessionService   │
│  - 调用 eleve-app::ProfileService   │
│  - 订阅 RuntimeEvent                │
└──────────────┬──────────────────────┘
               │ 直接调用
               ↓
┌─────────────────────────────────────┐
│  eleve-app (Service 层)             │
│  - SessionService                   │
│  - ProfileService                   │
│  - ToolService                      │
│  - ConfigService                    │
│  - 现有实现，transport-agnostic     │
└──────────────┬──────────────────────┘
               │ 调用
               ↓
┌─────────────────────────────────────┐
│  eleve-core + 其他 crates           │
│  - 核心逻辑                          │
│  - 现有实现                          │
└─────────────────────────────────────┘
```

### 2.3 为什么不需要 adapter 层？

1. **eleve-app 的 Service 已经提供了 transport-agnostic 的接口**
   - SessionService：会话管理（创建、列表、删除、发送消息）
   - ProfileService：Profile 管理
   - ToolService：工具管理
   - 这些接口已经是纯 Rust async 函数，没有 HTTP/WS 类型

2. **类型转换应该在 UI 组件内部做**
   - 不需要单独一层做转换
   - UI 组件直接消费 Service 返回的类型
   - 如果需要简化，在 UI 组件内部定义辅助函数

3. **保持架构极简，避免过度设计**
   - eleve-iced 只是 UI，不应该有业务逻辑
   - 所有业务逻辑都在 eleve-app

---

## 3. 目录结构

```
eleve-iced/
├── Cargo.toml
├── docs/                       # 设计 / 审查 / 计划文档
│   ├── architecture-design.md
│   ├── architecture-review.md
│   └── ui-development-plan.md
├── assets/icons/               # lucide 图标（SVG）
└── src/
    ├── main.rs                 # 入口：Iced application + 自绘窗体(chrome) + 状态壳 State
    └── ui/
        ├── mod.rs              # 数据模型 / Message 枚举 / update / view 顶层编排
        ├── theme.rs            # 颜色·尺寸·样式单一事实源
        ├── icon_bar.rs         # 左侧竖向导航（60px，9 导航项 + 设置/主题/关于簇）
        ├── left_panel.rs       # 左面板调度（Agents / File / Kanban / Cron / Tools / ...）
        ├── agents_panel.rs     # Agent 侧边栏（ProfilePanel + ProjectTreePanel + 新建弹窗）
        ├── chat_area.rs        # 中间聊天区（消息列表 + 输入框，常驻）
        ├── file_browser.rs     # 文件浏览器
        ├── kanban_panel.rs     # 看板
        ├── right_drawer.rs     # 右侧抽屉（4 个功能 tab，整体单卡片）
        ├── overlay.rs          # 模态弹窗（设置 / 主题 / 关于 / 模型选择）
        └── placeholder.rs      # 未实现 section 的占位视图
```

**设计原则**：
- 扁平结构，避免不必要的层级
- UI 组件按功能划分，不按技术划分
- 当前为纯 UI mock 阶段：状态壳在 `main.rs`，App 逻辑集中在 `ui/mod.rs`；`RuntimeEvent` 订阅（`events.rs`）推迟到接入 eleve-app 时再做

---

## 4. 核心模块设计

### 4.1 main.rs

```rust
use eleve_app::{AppService, SessionService, ProfileService};
use std::sync::Arc;

#[tokio::main]
async fn main() -> iced::Result {
    // 1. 初始化 AppService（参考 eleve-bin 的 main.rs）
    let app_service = initialize_app_service().await?;
    
    // 2. 创建 Service 实例
    let session_service = Arc::new(SessionService::new(app_service.clone()));
    let profile_service = Arc::new(ProfileService::new());
    
    // 3. 订阅 RuntimeEvent
    let event_rx = app_service.session_manager.subscribe_runtime_events().await;
    
    // 4. 启动 Iced 应用
    iced::application("ELEVE", App::update, App::view)
        .run_with(move || App::new(session_service, profile_service, event_rx))
}
```

### 4.2 app.rs

```rust
use std::sync::Arc;
use eleve_app::{SessionService, ProfileService};
use tokio::sync::broadcast::Receiver;
use eleve_core::RuntimeEvent;

pub struct App {
    session_service: Arc<SessionService>,
    profile_service: Arc<ProfileService>,
    event_rx: Receiver<RuntimeEvent>,
    state: AppState,
}

pub struct AppState {
    sessions: Vec<SessionInfo>,
    active_session: Option<String>,
    messages: Vec<MessageInfo>,
    input: String,
}

impl App {
    pub fn new(
        session_service: Arc<SessionService>,
        profile_service: Arc<ProfileService>,
        event_rx: Receiver<RuntimeEvent>,
    ) -> (Self, iced::Command<Message>) {
        // 初始化状态
        // 加载会话列表
        // 订阅事件
    }
    
    pub fn update(&mut self, message: Message) -> iced::Command<Message> {
        // 处理用户交互
        // 调用 Service 方法
    }
    
    pub fn view(&self) -> iced::Element<Message> {
        // 渲染 UI
    }
}
```

### 4.3 events.rs

```rust
use eleve_core::RuntimeEvent;
use tokio::sync::broadcast::Receiver;

pub async fn handle_events(
    mut event_rx: Receiver<RuntimeEvent>,
    event_tx: iced::futures::channel::mpsc::Sender<AppEvent>,
) {
    while let Ok(event) = event_rx.recv().await {
        // 将 RuntimeEvent 转换为 AppEvent
        // 发送给 Iced 的 update 循环
        let _ = event_tx.try_send(AppEvent::from_runtime(event));
    }
}
```

---

## 5. 依赖关系

### 5.1 直接依赖

```toml
[dependencies]
# UI 框架（实际采用 iced 0.14；0.13 仅为早期草稿建议，0.14 已稳定且 API 更顺手）
iced = { version = "0.14", features = ["tokio", "debug", "svg", "image"] }
iced-window-chrome = "0.1"   # 原生窗口边框/标题栏自绘，使窗体与背板融为一体

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 业务逻辑层（transport-agnostic，未来接入；当前纯 UI mock 阶段尚未依赖）
# eleve-app = { path = "../crates/eleve-app" }
# eleve-core = { path = "../crates/eleve-core" }

# 错误处理
anyhow = "1"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 5.2 不依赖的 crate

- `eleve-gateway`（HTTP/WS 服务，Iced 不需要）
- `eleve-cli`（CLI 工具，Iced 不需要）
- `eleve-agent`（Agent 实现，通过 eleve-app 间接使用）
- `eleve-tools-native`（工具实现，通过 eleve-app 间接使用）

---

## 6. 与现有系统的集成

### 6.1 数据共享

- **配置文件**：共享 `~/.eleve/config.yaml`
- **会话数据**：共享 `~/.eleve/sessions/`
- **Profile 配置**：共享 `~/.eleve/profiles/`

### 6.2 互不干扰

- **eleved.exe**：继续独立运行，给 Tauri/CLI/Web 用
- **eleve-iced.exe**：内嵌引擎，单进程运行
- **数据存储**：使用相同的 SQLite 数据库（eleve-store 的 redb）

### 6.3 并行开发

- Tauri 版本继续迭代，不受影响
- Iced 版本独立开发，独立测试
- 未来可共存，用户选择使用哪个

---

## 7. 开发计划

### Phase 1：基础框架（1 周）

- [ ] 初始化 AppService（参考 eleve-bin 的 main.rs）
- [ ] 实现基本的 Iced App 结构
- [ ] 1+3 布局骨架（Sidebar + Chat + Drawer）
- [ ] 编译通过，窗口能弹出来

### Phase 2：核心功能（2 周）

- [ ] 会话列表（调用 SessionService::list_sessions）
- [ ] 消息流式渲染（订阅 RuntimeEvent）
- [ ] 输入框 + 发送功能（调用 SessionService::send_message）
- [ ] 工具调用展示

### Phase 3：完善体验（2 周）

- [ ] Markdown 渲染（代码高亮、表格、链接）
- [ ] Profile 切换（调用 ProfileService）
- [ ] 设置页面（调用 ConfigService）
- [ ] 快捷键系统

### Phase 4：生产就绪（1 周）

- [ ] 性能优化（虚拟列表、图片缓存）
- [ ] 打包部署（Windows installer）
- [ ] 内存测试（目标 < 200MB）
- [ ] 文档

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Iced 0.13 API 不稳定 | 中 | 锁定版本，不跟进 minor 更新 |
| eleve-app API 变化 | 中 | UI 层隔离，变化只影响调用点 |
| Markdown 渲染性能 | 中 | 增量渲染 + 虚拟列表 |
| 内存泄漏 | 中 | 定期 profile，设置缓存上限 |
| 引擎线程阻塞 UI | 高 | AppService 跑在独立 tokio runtime |

---

## 9. 待讨论问题

### 9.1 架构层面

1. **AppService 初始化**：
   - 需要初始化哪些组件？
   - 是否需要完整的 bootstrap 流程？
   - 建议：参考 eleve-bin 的 main.rs，最小化初始化

2. **事件订阅机制**：
   - RuntimeEvent 是否包含所有 UI 需要的事件？
   - 是否需要扩展 RuntimeEvent？
   - 建议：先评估现有事件，不足时再扩展

### 9.2 功能层面

3. **多会话支持**：同时开多个会话？
   - Tauri 版本支持，Iced 是否也要？
   - 建议：Phase 1 只支持单会话，Phase 3 再加

4. **插件系统**：需要支持 WASM 插件吗？
   - eleve-plugin 已有 WASM 支持
   - 建议：Phase 1 不接插件，先跑通核心流程

### 9.3 技术层面

5. **Iced 版本**：0.13 vs 0.14？
   - 0.13 稳定，0.14 已发布且 API 更顺手（如 `Border.radius` 改为 `Radius` 类型、新增 `container::Style.shadow` 可画发光环）
   - 决定：采用 0.14（已实现并零警告编译通过）；若后续发现 0.14 回归可回退 0.13

6. **渲染后端**：wgpu (Vulkan/DX12) vs tiny-skia (CPU)？
   - wgpu 性能好，但内存占用高
   - tiny-skia 内存低，但复杂场景卡顿
   - 建议：默认 wgpu，提供 tiny-skia fallback

---

## 10. 下一步

1. 讨论上述待讨论问题
2. 确认架构设计
3. 开始 Phase 1 开发

---

**附录：参考资源**

- [Iced 官方文档](https://docs.rs/iced/0.14/iced/)
- [Iced 示例库](https://github.com/iced-rs/iced/tree/master/examples)
- [ELEVE 架构分析](./architecture-analysis.md)
- [eleve-app::AppService](../crates/eleve-app/src/app_service.rs)
- [eleve-app::SessionService](../crates/eleve-app/src/session_service.rs)
- [eleve-core::RuntimeEvent](../crates/eleve-core/src/lib.rs)
