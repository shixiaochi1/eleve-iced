mod agents_panel;
mod chat_area;
mod file_browser;
mod grid_view;
mod icon_bar;
mod kanban_panel;
mod left_panel;
mod overlay;
mod placeholder;
mod right_drawer;
pub mod theme;

use std::collections::{HashMap, HashSet};

use iced::advanced::widget::{operate, operation::scrollable::snap_to};
use iced::widget::{container, row, stack, Id};
use iced::{Element, Length, Padding, Background, Subscription};
use iced::time;
use std::time::Duration;

/// Agents 面板滚动容器 id（新建卡片后用于 snap_to 顶部，确保新卡片立即可见）
pub const AGENTS_PANEL_SCROLL_ID: &str = "agents-panel-scroll";

/// 聊天消息列表滚动容器 id（新消息后自动滚到底部）
pub const CHAT_SCROLL_ID: &str = "chat-scroll";

// ============================================================
// 导航分区 —— 严格对齐 Tauri 的三类「弹出」形态
//   1) 左侧面板 (SidePanel)   : 聊天区常驻，左侧出现分区内容
//   2) 右侧抽屉 (Pane right)  : 4 个 tab（文件/终端/预览/产物）
//   3) 模态弹窗 (OverlayView) : 全屏居中 + 暗化背景
// ============================================================

/// 左侧面板 section（点图标栏 → 在聊天区左侧出现对应内容；再点关闭）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanel {
    Agents,
    Kanban,
    Gateway,
    Channels,
    Cron,
    Tools,
    Learning,
    Debug,
    Usage,
}

/// 右侧抽屉 tab（文件浏览器 / 终端 / 预览 / 产物）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightTab {
    Files,
    Terminal,
    Preview,
    Artifacts,
}

/// 模态弹窗类型（设置 / 主题 / 关于 / 模型选择）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Settings,
    Theme,
    About,
    Model,
}

/// 新建弹窗类型（对齐 Tauri CreateAgentDialog / ProjectDialog）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateDialog {
    Agent,
    Project,
}

/// 主视图模式：单视图（默认聊天） / 宫格多 Agent 视图（对齐 Tauri GridModeView）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Single,
    Grid,
}

// ============================================================
// 数据模型（mock）
// ============================================================

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant"
    pub blocks: Vec<ChatBlock>,
}

/// 聊天消息块：纯文本 / 代码块 / 工具调用卡片
#[derive(Debug, Clone)]
pub enum ChatBlock {
    Text(String),
    Code {
        language: Option<String>,
        code: String,
    },
    ToolCall {
        name: String,
        status: ToolStatus,
        result: String,
    },
    /// AI 思考过程（可折叠）。summary 为折叠态的一句话摘要，detail 为展开后的完整思考内容（Markdown）。
    Thinking {
        summary: String,
        detail: String,
    },
}

/// 流式输出状态：把 full 逐步 reveal 到 shown 个字符
#[derive(Debug, Clone)]
pub struct Streaming {
    pub full: String,
    pub shown: usize, // 已显示的「字符数」（非字节）
}

/// 工具调用状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // mock 阶段可能未全部展示，保留给真实后端状态
pub enum ToolStatus {
    Running,
    Done,
    Error,
}

impl ToolStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ToolStatus::Running => "执行中",
            ToolStatus::Done => "已完成",
            ToolStatus::Error => "失败",
        }
    }
    pub fn accent(&self) -> iced::Color {
        match self {
            ToolStatus::Running => iced::Color::from_rgb(0.85, 0.62, 0.20),
            ToolStatus::Done => iced::Color::from_rgb(0.30, 0.70, 0.45),
            ToolStatus::Error => iced::Color::from_rgb(0.86, 0.27, 0.27),
        }
    }
}

/// Agent 身份（对齐 Tauri ProfilePanel.ProfileCardData）
#[derive(Debug, Clone)]
#[allow(dead_code)] // has_avatar / avatar_key 为真实头像能力字段，mock 阶段暂以 glyph 渲染
pub struct AgentProfile {
    pub id: String,
    pub display_name: String,
    /// Agent 主题色（#RRGGBB；来自后端 profile.yaml color，仅 UI 用）
    pub color: Option<String>,
    /// 是否有上传头像（有图显示图，无图显示首字母 glyph）
    pub has_avatar: bool,
    /// 默认头像 key（预设头像库，随主题色渲染 SVG）
    pub avatar_key: Option<String>,
    pub is_default: bool,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub skill_count: usize,
}

/// 会话预览（对齐 Tauri SessionPreview）
#[derive(Debug, Clone)]
#[allow(dead_code)] // id 为模型身份字段，视图暂未展示
pub struct SessionPreview {
    pub id: String,
    pub title: String,
    /// 已格式化为 "刚刚" / "3m" / "2h" / "8/17"
    pub last_active: String,
}

/// Lane（对齐 Tauri LaneGroup，钻取视图用）
#[derive(Debug, Clone)]
#[allow(dead_code)] // id 为模型身份字段，视图暂未展示
pub struct LaneGroup {
    pub id: String,
    pub label: String,
    pub session_count: usize,
    pub sessions: Vec<SessionPreview>,
}

/// Repo（对齐 Tauri RepoNode，钻取视图用）
#[derive(Debug, Clone)]
#[allow(dead_code)] // id 为模型身份字段，视图暂未展示
pub struct RepoNode {
    pub id: String,
    pub label: String,
    pub session_count: usize,
    pub lanes: Vec<LaneGroup>,
}

/// 项目（对齐 Tauri ProjectNode，overview / drill 共用）
#[derive(Debug, Clone)]
pub struct ProjectNode {
    pub id: String,
    pub label: String,
    /// 数据模型字段：对齐 Tauri ProjectNode.path（用于钻取/外部打开仓库）。
    /// 侧边栏卡片本身不渲染路径，故仅参与数据构造、暂不读取。
    #[allow(dead_code)]
    pub path: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_auto: bool,
    /// Home 桶（无归属会话兜底；恒首、无右键菜单）
    pub is_no_project: bool,
    pub session_count: usize,
    /// 卡片右侧时间（已格式化 "刚刚"/"3m"/"2h"/"8/17"；Home 桶为空）
    pub last_active: String,
    /// 总览模式预览会话（Top3，对齐 Hermes PROJECT_PREVIEW_COUNT）
    pub preview_sessions: Vec<SessionPreview>,
    /// 钻取模式全量水合（Repo → Lane → Session）
    pub repos: Vec<RepoNode>,
}

#[derive(Debug, Clone)]
pub struct FsNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FsNode>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardStatus {
    Todo,
    Doing,
    Review,
    Done,
}

impl CardStatus {
    pub fn label(&self) -> &'static str {
        match self {
            CardStatus::Todo => "待办",
            CardStatus::Doing => "进行中",
            CardStatus::Review => "评审",
            CardStatus::Done => "完成",
        }
    }
    pub fn accent(&self) -> iced::Color {
        match self {
            CardStatus::Todo => iced::Color::from_rgb(0.55, 0.58, 0.62),
            CardStatus::Doing => iced::Color::from_rgb(0.38, 0.53, 0.89),
            CardStatus::Review => iced::Color::from_rgb(0.85, 0.62, 0.20),
            CardStatus::Done => iced::Color::from_rgb(0.30, 0.70, 0.45),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KanbanCard {
    pub id: String,
    pub title: String,
    pub status: CardStatus,
    pub model: String,
    pub agent: String,
}

#[derive(Debug, Clone)]
pub struct KanbanColumn {
    pub id: String,
    pub title: String,
    pub cards: Vec<KanbanCard>,
}

// ============================================================
// 消息
// ============================================================

#[derive(Debug, Clone)]
pub enum Message {
    // ── 导航 / 弹窗 ──
    ToggleLeftPanel(LeftPanel), // 图标栏点击 → 左面板开关（toggle）
    ToggleFiles,                // 文件图标 → 右侧抽屉（开/关，定位到 Files tab）
    OpenRightTab(RightTab),     // 右抽屉内切换 tab
    CloseRight,                 // 关闭右抽屉
    OpenOverlay(Overlay),       // 设置/主题/关于/模型 → 模态弹窗
    CloseOverlay,               // 关闭模态弹窗（ESC / 点背景）
    Dismiss,                    // 点击弹窗内部（非背景）→ 不关闭（阻止事件穿透）

    // ── 聊天区 ──
    InputChanged(String),
    SendPressed,
    LinkClicked(String),   // Markdown 链接点击
    ToggleThinking(usize), // 折叠/展开某条消息的思考过程（按消息下标）
    StreamTick,            // 流式输出推进
    CaretTick,             // 流式光标闪烁
    Noop,

    // ── Agent 左侧栏 ──
    SelectProfile(String),
    ToggleProjectExpand(String), // 展开/折叠项目预览会话（overview）
    SelectProject(String),       // 选中（激活）项目
    EnterDrill(String),          // 进入项目钻取视图（Repo/Lane/Session 树）
    ExitDrill,                   // 返回项目总览
    DeleteProfile(String),       // 删除 Agent（mock：直接移除，无确认）
    DeleteProject(String),       // 删除/忽略项目（自动项目 = dismiss，显式项目 = 删除）
    OpenCreateDialog(CreateDialog),
    CloseCreateDialog,
    CreateInputChanged(String),
    ConfirmCreate,

    // ── 文件浏览器（右抽屉）──
    ToggleDir(String),
    SelectFile(String),

    // ── 看板（左面板）──
    MoveCard(String, String), // (card_id, target_column_id)

    // ── 设置开关（模态弹窗 mock）──
    ToggleSetting(String),

    // ── 主题（对齐 Tauri 外观 / accent / 字号）──
    SetAccent(String),
    SetAppearance(theme::Appearance),
    SetFontScale(theme::FontScale),

    // ── 主视图模式切换（单视图 / 宫格，对齐 Tauri GridModeView）──
    SetViewMode(ViewMode),
    /// 宫格卡片「展开」：选中该 Agent 并回到单视图
    GridExpand(String),
}

// ============================================================
// 状态
// ============================================================

pub struct State {
    // 布局
    pub active_panel: Option<LeftPanel>, // 左面板（None = 关闭）
    pub right_open: bool,                // 右抽屉是否展开
    pub right_tab: RightTab,             // 右抽屉当前 tab
    pub overlay: Option<Overlay>,         // 模态弹窗

    // 聊天区
    pub input: String,
    pub messages: Vec<ChatMessage>,
    // 流式输出
    pub streaming: Option<Streaming>,
    pub caret_on: bool,
    // 思考过程折叠（按消息下标）
    pub thinking_collapsed: HashSet<usize>,

    // Agent 左侧栏
    pub profiles: Vec<AgentProfile>,
    pub selected_profile: String,
    pub projects: Vec<ProjectNode>,
    pub expanded_projects: HashSet<String>, // 按 id（overview 展开态）
    pub selected_project: Option<String>,   // 激活项目 id（含 __no_project__）
    pub active_session: Option<String>,     // 激活会话 id（预览/钻取中橙色高亮，对齐 Tauri accent-orange）
    pub drill_project: Option<String>,      // 钻取视图中的项目 id
    pub create_dialog: Option<CreateDialog>,
    pub create_input: String,

    // 文件浏览器
    pub fs_root_name: String,
    pub fs_nodes: Vec<FsNode>,
    pub expanded_dirs: HashSet<String>,
    pub selected_file: Option<String>,

    // 看板
    pub kanban_columns: Vec<KanbanColumn>,

    // 设置开关
    pub settings: HashMap<String, bool>,

    // 主题（对齐 Tauri appearance / accent / 字号）
    pub accent: String,
    pub appearance: theme::Appearance,
    pub font_scale: theme::FontScale,

    // 主视图模式（单视图 / 宫格）
    pub view_mode: ViewMode,
}

fn initial_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: "user".into(),
            blocks: vec![ChatBlock::Text("你好，请介绍一下你自己".into())],
        },
        ChatMessage {
            role: "assistant".into(),
            blocks: parse_message_content(
                "你好！我是 **ELEVE Agent**，一个基于 `Rust` 构建的 AI 智能体。\n\n我可以帮你做这些事：\n\n- 编写与重构代码\n- 分析数据并生成图表\n- 自动化重复任务\n\n更多能力见 [官方文档](https://eleve.agent.dev)。有什么我可以帮你的吗？",
            ),
        },
        ChatMessage {
            role: "user".into(),
            blocks: vec![ChatBlock::Text("帮我把 eleve-iced 的布局按 Tauri 的方式重构一下".into())],
        },
        ChatMessage {
            role: "assistant".into(),
            blocks: vec![
                ChatBlock::Thinking {
                    summary: "分析 Tauri 的三类弹出形态，决定 iced 前端的布局调度策略。".into(),
                    detail: "需要先梳理 Tauri 的导航模型：\n\n1. **左侧面板** —— 聊天区常驻，点击图标栏在左侧出现分区内容。\n2. **右侧抽屉** —— 文件 / 终端 / 预览 / 产物四个 tab。\n3. **模态弹窗** —— 全屏居中 + 暗化背景（设置 / 主题 / 关于）。\n\n据此，iced 端应当保留「主聊天区」为核心，其余均为覆盖层。".into(),
                },
                ChatBlock::Text(
                    "Tauri 的设计是：主聊天区永远常驻，左侧图标栏的点击分为三类——打开左侧面板、切换右侧抽屉、弹出模态窗口。".into(),
                ),
            ],
        },
        ChatMessage {
            role: "assistant".into(),
            blocks: parse_message_content(
                "我会先读取当前项目结构：\n\n```rust\nlet layout = Layout::new()\n    .with_sidebar(true)\n    .with_right_drawer(true);\n```\n\n然后逐步重构 iced 前端。",
            ),
        },
        ChatMessage {
            role: "assistant".into(),
            blocks: vec![
                ChatBlock::ToolCall {
                    name: "file_read".into(),
                    status: ToolStatus::Done,
                    result: "已读取 src/ui/mod.rs（当前包含 1+3 布局调度）".into(),
                },
                ChatBlock::Text("项目结构确认完毕，开始重构。".into()),
            ],
        },
    ]
}

/// 将一段纯文本按 Markdown 代码围栏拆分为 Text / Code 块。
/// 暂不支持嵌套围栏；工具调用块由构造者显式创建。
pub fn parse_message_content(text: &str) -> Vec<ChatBlock> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_code = false;
    let mut language: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if !in_code && trimmed.starts_with("```") {
            if !current.trim().is_empty() {
                blocks.push(ChatBlock::Text(current.trim_end().into()));
                current.clear();
            }
            in_code = true;
            let lang = trimmed.trim_start_matches('`').trim();
            language = if lang.is_empty() { None } else { Some(lang.into()) };
        } else if in_code && trimmed.starts_with("```") {
            blocks.push(ChatBlock::Code {
                language: language.take(),
                code: current.trim_end().into(),
            });
            current.clear();
            in_code = false;
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }

    if in_code {
        blocks.push(ChatBlock::Code {
            language: language.take(),
            code: current.trim_end().into(),
        });
    } else if !current.trim().is_empty() {
        blocks.push(ChatBlock::Text(current.trim_end().into()));
    }

    blocks
}

/// 在系统默认浏览器中打开 URL（用于 Markdown 链接点击）。
fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .status();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).status();
    }
}

/// 流式输出 + 光标闪烁订阅：仅在 streaming 进行中激活。
pub fn streaming_subscription(state: &State) -> Subscription<Message> {
    if state.streaming.is_some() {
        let stream = time::every(Duration::from_millis(24)).map(|_| Message::StreamTick);
        let caret = time::every(Duration::from_millis(530)).map(|_| Message::CaretTick);
        Subscription::batch([stream, caret])
    } else {
        Subscription::none()
    }
}

fn initial_profiles() -> Vec<AgentProfile> {
    use AgentProfile as A;
    vec![
        A {
            id: "default".into(),
            display_name: "默认 Agent".into(),
            color: None,
            has_avatar: false,
            avatar_key: None,
            is_default: true,
            model: Some("qwen3.7-plus".into()),
            provider: Some("siliconflow".into()),
            skill_count: 6,
        },
        A {
            id: "coder".into(),
            display_name: "代码助手".into(),
            color: Some("#3B82F6".into()),
            has_avatar: false,
            avatar_key: None,
            is_default: false,
            model: Some("deepseek-coder".into()),
            provider: Some("deepseek".into()),
            skill_count: 9,
        },
        A {
            id: "analyst".into(),
            display_name: "数据分析师".into(),
            color: Some("#10B981".into()),
            has_avatar: false,
            avatar_key: None,
            is_default: false,
            model: Some("qwen3.7-plus".into()),
            provider: Some("siliconflow".into()),
            skill_count: 4,
        },
        A {
            id: "writer".into(),
            display_name: "文档写手".into(),
            color: Some("#F59E0B".into()),
            has_avatar: false,
            avatar_key: None,
            is_default: false,
            model: None,
            provider: None,
            skill_count: 3,
        },
        A {
            id: "ops".into(),
            display_name: "运维助手".into(),
            color: Some("#EF4444".into()),
            has_avatar: false,
            avatar_key: None,
            is_default: false,
            model: Some("qwen3.7-plus".into()),
            provider: Some("siliconflow".into()),
            skill_count: 7,
        },
    ]
}

fn initial_projects() -> Vec<ProjectNode> {
    use ProjectNode as P;
    use RepoNode as R;
    use LaneGroup as L;
    use SessionPreview as S;

    let home_sessions = vec![
        S { id: "s-h1".into(), title: "未归类的临时会话".into(), last_active: "刚刚".into() },
        S { id: "s-h2".into(), title: "快速问答：Rust 生命周期".into(), last_active: "3m".into() },
        S { id: "s-h3".into(), title: "对比 iced 与 egui".into(), last_active: "1h".into() },
    ];

    let core_sessions = vec![
        S { id: "s-c1".into(), title: "重构布局为 1+3 卡片".into(), last_active: "刚刚".into() },
        S { id: "s-c2".into(), title: "对齐 Tauri 弹窗逻辑".into(), last_active: "12m".into() },
        S { id: "s-c3".into(), title: "修复右侧抽屉标题栏".into(), last_active: "2h".into() },
    ];

    let app_sessions = vec![
        S { id: "s-a1".into(), title: "接入 session_service".into(), last_active: "5m".into() },
        S { id: "s-a2".into(), title: "profile_service 单元测试".into(), last_active: "8/16".into() },
    ];

    vec![
        // Home 桶（无归属会话兜底；恒首、无计数/时间、无右键菜单）
        P {
            id: "__no_project__".into(),
            label: "主目录".into(),
            path: Some("/workspace".into()),
            color: None,
            icon: None,
            is_auto: false,
            is_no_project: true,
            session_count: home_sessions.len(),
            last_active: String::new(),
            preview_sessions: home_sessions,
            repos: vec![],
        },
        // 显式项目：eleve-core（带主题色，含钻取 Repo/Lane/Session 树）
        P {
            id: "eleve-core".into(),
            label: "eleve-core".into(),
            path: Some("/projects/eleve-core".into()),
            color: Some("#3B82F6".into()),
            icon: None,
            is_auto: false,
            is_no_project: false,
            session_count: core_sessions.len(),
            last_active: "2h".into(),
            preview_sessions: core_sessions.clone(),
            repos: vec![
                R {
                    id: "eleve-core@main".into(),
                    label: "eleve-core @ main".into(),
                    session_count: 3,
                    lanes: vec![
                        L {
                            id: "lane-main".into(),
                            label: "main".into(),
                            session_count: 2,
                            sessions: core_sessions.clone(),
                        },
                        L {
                            id: "lane-feat".into(),
                            label: "feat/tauri-layout".into(),
                            session_count: 1,
                            sessions: vec![
                                S { id: "s-c4".into(), title: "Tauri 式布局分支".into(), last_active: "1h".into() },
                            ],
                        },
                    ],
                },
            ],
        },
        // 显式项目：eleve-app
        P {
            id: "eleve-app".into(),
            label: "eleve-app".into(),
            path: Some("/projects/eleve-app".into()),
            color: Some("#10B981".into()),
            icon: None,
            is_auto: false,
            is_no_project: false,
            session_count: app_sessions.len(),
            last_active: "8/16".into(),
            preview_sessions: app_sessions.clone(),
            repos: vec![
                R {
                    id: "eleve-app@main".into(),
                    label: "eleve-app @ main".into(),
                    session_count: 2,
                    lanes: vec![
                        L {
                            id: "lane-app-main".into(),
                            label: "main".into(),
                            session_count: 2,
                            sessions: app_sessions.clone(),
                        },
                    ],
                },
            ],
        },
        // 自动项目（自动发现，无 db 记录；菜单为 收养/移除）
        P {
            id: "eleve-gateway".into(),
            label: "eleve-gateway".into(),
            path: Some("/projects/eleve-gateway".into()),
            color: None,
            icon: None,
            is_auto: true,
            is_no_project: false,
            session_count: 0,
            last_active: "—".into(),
            preview_sessions: vec![],
            repos: vec![],
        },
    ]
}

fn initial_fs() -> Vec<FsNode> {
    use FsNode as F;
    vec![
        F { name: "src".into(), path: "/src".into(), is_dir: true, children: Some(vec![
            F { name: "main.rs".into(), path: "/src/main.rs".into(), is_dir: false, children: None },
            F { name: "ui".into(), path: "/src/ui".into(), is_dir: true, children: Some(vec![
                F { name: "mod.rs".into(), path: "/src/ui/mod.rs".into(), is_dir: false, children: None },
                F { name: "chat_area.rs".into(), path: "/src/ui/chat_area.rs".into(), is_dir: false, children: None },
            ]) },
        ]) },
        F { name: "Cargo.toml".into(), path: "/Cargo.toml".into(), is_dir: false, children: None },
        F { name: "README.md".into(), path: "/README.md".into(), is_dir: false, children: None },
        F { name: "assets".into(), path: "/assets".into(), is_dir: true, children: Some(vec![
            F { name: "icons".into(), path: "/assets/icons".into(), is_dir: true, children: Some(vec![
                F { name: "Elogo.svg".into(), path: "/assets/icons/Elogo.svg".into(), is_dir: false, children: None },
            ]) },
        ]) },
    ]
}

fn initial_kanban() -> Vec<KanbanColumn> {
    use CardStatus::*;
    vec![
        KanbanColumn {
            id: "todo".into(),
            title: "待办".into(),
            cards: vec![
                KanbanCard { id: "c1".into(), title: "实现文件浏览器虚拟滚动".into(), status: Todo, model: "qwen3.7-plus".into(), agent: "coder".into() },
                KanbanCard { id: "c2".into(), title: "接入 Subagent 监控面板".into(), status: Todo, model: "deepseek".into(), agent: "default".into() },
                KanbanCard { id: "c3".into(), title: "看板拖拽交互对齐 Hermes".into(), status: Todo, model: "qwen3.7-plus".into(), agent: "ops".into() },
            ],
        },
        KanbanColumn {
            id: "doing".into(),
            title: "进行中".into(),
            cards: vec![
                KanbanCard { id: "c4".into(), title: "Markdown 代码高亮渲染".into(), status: Doing, model: "qwen3.7-plus".into(), agent: "coder".into() },
                KanbanCard { id: "c5".into(), title: "上下文条 15s 降频轮询".into(), status: Doing, model: "deepseek".into(), agent: "default".into() },
            ],
        },
        KanbanColumn {
            id: "review".into(),
            title: "评审".into(),
            cards: vec![
                KanbanCard { id: "c6".into(), title: "Tauri 文件树抖动根治".into(), status: Review, model: "qwen3.7-plus".into(), agent: "coder".into() },
            ],
        },
        KanbanColumn {
            id: "done".into(),
            title: "完成".into(),
            cards: vec![
                KanbanCard { id: "c7".into(), title: "1+3 布局骨架".into(), status: Done, model: "qwen3.7-plus".into(), agent: "default".into() },
                KanbanCard { id: "c8".into(), title: "图标栏对齐 Tauri".into(), status: Done, model: "deepseek".into(), agent: "default".into() },
            ],
        },
    ]
}

fn initial_settings() -> HashMap<String, bool> {
    let mut m = HashMap::new();
    m.insert("auto_update".into(), true);
    m.insert("send_on_enter".into(), true);
    m.insert("stream_tokens".into(), true);
    m.insert("compact_panel".into(), false);
    m.insert("sound_alert".into(), false);
    m
}

// ============================================================
// Iced Application 接口
// ============================================================

pub fn new() -> (State, iced::Task<Message>) {
    let state = State {
        active_panel: Some(LeftPanel::Agents),
        right_open: false,
        right_tab: RightTab::Files,
        overlay: None,
        input: String::new(),
        messages: initial_messages(),
        streaming: None,
        caret_on: true,
        thinking_collapsed: HashSet::new(),
        profiles: initial_profiles(),
        selected_profile: "default".into(),
        projects: initial_projects(),
        expanded_projects: {
            let mut s = HashSet::new();
            s.insert("eleve-core".to_string());
            s.insert("__no_project__".to_string());
            s
        },
        selected_project: Some("eleve-core".into()),
        active_session: Some("s-c1".into()),
        drill_project: None,
        create_dialog: None,
        create_input: String::new(),
        fs_root_name: "eleve-iced".into(),
        fs_nodes: initial_fs(),
        expanded_dirs: {
            let mut s = HashSet::new();
            s.insert("/src".to_string());
            s
        },
        selected_file: None,
        kanban_columns: initial_kanban(),
        settings: initial_settings(),
        accent: theme::DEFAULT_ACCENT.to_string(),
        appearance: theme::DEFAULT_APPEARANCE,
        font_scale: theme::DEFAULT_FONT_SCALE,
        view_mode: ViewMode::Single,
    };
    // 初始化全局调色板（默认 = 石墨灰 + 深色）
    theme::apply(&state.accent, state.appearance, state.font_scale);
    (state, iced::Task::none())
}

fn find_card(columns: &mut Vec<KanbanColumn>, card_id: &str) -> Option<KanbanCard> {
    for col in columns.iter_mut() {
        if let Some(pos) = col.cards.iter().position(|c| c.id == card_id) {
            return Some(col.cards.remove(pos));
        }
    }
    None
}

pub fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        // ── 左面板：toggle（再次点击同一个 → 关闭）──
        Message::ToggleLeftPanel(panel) => {
            state.active_panel = if state.active_panel == Some(panel) {
                None
            } else {
                Some(panel)
            };
        }
        // ── 文件图标：切换右抽屉（定位到 Files tab）──
        Message::ToggleFiles => {
            if state.right_open && state.right_tab == RightTab::Files {
                state.right_open = false;
            } else {
                state.right_open = true;
                state.right_tab = RightTab::Files;
            }
        }
        Message::OpenRightTab(tab) => {
            state.right_open = true;
            state.right_tab = tab;
        }
        Message::CloseRight => {
            state.right_open = false;
        }
        // ── 模态弹窗 ──
        Message::OpenOverlay(o) => {
            state.overlay = Some(o);
        }
        Message::CloseOverlay => {
            state.overlay = None;
        }
        Message::Dismiss => {
            // 点击弹窗内部：不关闭（阻止事件穿透到背景）
        }
        // ── 聊天 ──
        Message::InputChanged(s) => {
            state.input = s;
        }
        Message::SendPressed => {
            let text = state.input.trim().to_string();
            if !text.is_empty() {
                let user_blocks = parse_message_content(&text);
                state.messages.push(ChatMessage { role: "user".into(), blocks: user_blocks });
                // 占位 assistant 消息，随后由流式订阅逐步填充
                state.messages.push(ChatMessage {
                    role: "assistant".into(),
                    blocks: vec![ChatBlock::Text(String::new())],
                });
                let reply = format!(
                    "（演示）已收到你的消息：「{}」。\n\n这是一段**模拟回复**，后端接入后将由真实 Agent 生成。支持：\n\n- **Markdown** 行内样式\n- `行内代码`\n- [可点击链接](https://eleve.agent.dev)\n\n流式输出动画正在演示中…",
                    text
                );
                state.streaming = Some(Streaming { full: reply, shown: 0 });
                state.caret_on = true;
                state.input.clear();
                return operate(snap_to(
                    Id::new(CHAT_SCROLL_ID),
                    iced::widget::scrollable::RelativeOffset::END.into(),
                ));
            }
        }
        // ── 聊天：链接 / 思考折叠 / 流式 ──
        Message::LinkClicked(url) => {
            let url = url.clone();
            return iced::Task::perform(async move { open_url(&url); }, |_| Message::Noop);
        }
        Message::ToggleThinking(idx) => {
            if state.thinking_collapsed.contains(&idx) {
                state.thinking_collapsed.remove(&idx);
            } else {
                state.thinking_collapsed.insert(idx);
            }
        }
        Message::StreamTick => {
            if let Some(s) = &mut state.streaming {
                let total = s.full.chars().count();
                let step = 3usize;
                if s.shown + step >= total {
                    let full = s.full.clone();
                    state.messages.last_mut().map(|m| m.blocks = parse_message_content(&full));
                    state.streaming = None;
                    state.caret_on = false;
                } else {
                    s.shown += step;
                    let mut partial: String = s.full.chars().take(s.shown).collect();
                    if state.caret_on {
                        partial.push('▍');
                    }
                    state.messages.last_mut().map(|m| m.blocks = parse_message_content(&partial));
                }
                return operate(snap_to(
                    Id::new(CHAT_SCROLL_ID),
                    iced::widget::scrollable::RelativeOffset::END.into(),
                ));
            }
        }
        Message::CaretTick => {
            if state.streaming.is_some() {
                state.caret_on = !state.caret_on;
                if let Some(s) = &state.streaming {
                    let mut partial: String = s.full.chars().take(s.shown).collect();
                    if state.caret_on {
                        partial.push('▍');
                    }
                    state.messages.last_mut().map(|m| m.blocks = parse_message_content(&partial));
                }
                return operate(snap_to(
                    Id::new(CHAT_SCROLL_ID),
                    iced::widget::scrollable::RelativeOffset::END.into(),
                ));
            }
        }
        Message::Noop => {}

        // ── Agent 左侧栏 ──
        Message::SelectProfile(id) => {
            state.selected_profile = id;
        }
        Message::ToggleProjectExpand(id) => {
            if state.expanded_projects.contains(&id) {
                state.expanded_projects.remove(&id);
            } else {
                state.expanded_projects.insert(id);
            }
        }
        Message::SelectProject(id) => {
            // 🔴 点选 = 纯前端激活（对齐 Tauri：高亮不被刷新回跳）
            state.selected_project = Some(id.clone());
            // 激活项目首条预览会话 → 橙色高亮（对齐 Tauri 预览区 active 行）
            state.active_session = state
                .projects
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| p.preview_sessions.first())
                .map(|s| s.id.clone());
        }
        Message::EnterDrill(id) => {
            state.drill_project = Some(id);
        }
        Message::ExitDrill => {
            state.drill_project = None;
        }
        Message::DeleteProfile(id) => {
            state.profiles.retain(|p| p.id != id);
            // 始终保证有一个默认选中的 Agent 卡片（防止选中态悬空 → 视觉空白）。
            // 默认 Agent 不可删（UI 上无删除按钮），故优先回退到它；否则回退到首个。
            if !state.profiles.iter().any(|p| p.id == state.selected_profile) {
                state.selected_profile = if state.profiles.iter().any(|p| p.id == "default") {
                    "default".into()
                } else {
                    state.profiles.first().map(|p| p.id.clone()).unwrap_or_default()
                };
            }
        }
        Message::DeleteProject(id) => {
            // 自动发现项目 = 忽略（dismiss）；显式项目 = 删除
            state.projects.retain(|p| p.id != id);
            // 选中态悬空 → 回退到仍存在的项目（主目录桶恒在，必有一个默认项目显示）
            if !state.projects.iter().any(|p| Some(&p.id) == state.selected_project.as_ref()) {
                state.selected_project = state.projects.first().map(|p| p.id.clone());
            }
            if state.drill_project.as_deref() == Some(id.as_str()) {
                state.drill_project = None;
            }
        }
        Message::OpenCreateDialog(kind) => {
            state.create_dialog = Some(kind);
            state.create_input.clear();
        }
        Message::CloseCreateDialog => {
            state.create_dialog = None;
            state.create_input.clear();
        }
        Message::CreateInputChanged(s) => {
            state.create_input = s;
        }
        Message::ConfirmCreate => {
            let name = state.create_input.trim().to_string();
            // 仅当名称非空才真正创建（空名称 = 无操作）
            let created = !name.is_empty();
            if created {
                match state.create_dialog {
                    Some(CreateDialog::Agent) => {
                        // 置顶插入：新建 Agent 立即可见（不再沉到列表底部被遮挡）
                        state.profiles.insert(
                            0,
                            AgentProfile {
                                id: name.clone(),
                                display_name: name.clone(),
                                color: None,
                                has_avatar: false,
                                avatar_key: None,
                                is_default: false,
                                model: None,
                                provider: None,
                                skill_count: 0,
                            },
                        );
                        state.selected_profile = name;
                    }
                    Some(CreateDialog::Project) => {
                        let proj = ProjectNode {
                            id: name.clone(),
                            label: name.clone(),
                            path: Some(format!("/projects/{}", name)),
                            color: None,
                            icon: None,
                            is_auto: false,
                            is_no_project: false,
                            session_count: 0,
                            last_active: "刚刚".into(),
                            preview_sessions: vec![],
                            repos: vec![],
                        };
                        // 置顶于「主目录」桶之后（保持 Home 恒首），新项目立即可见
                        let idx = state
                            .projects
                            .iter()
                            .position(|p| p.is_no_project)
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        state.projects.insert(idx, proj);
                        state.selected_project = Some(name.clone());
                        // 新建项目默认展开，立刻看到空态/预览
                        state.expanded_projects.insert(name);
                    }
                    None => {}
                }
            }
            state.create_dialog = None;
            state.create_input.clear();

            // 创建成功后把面板滚动到顶部，确保新卡片“弹”到可见区域
            if created {
                return operate(snap_to(
                    Id::new(AGENTS_PANEL_SCROLL_ID),
                    iced::widget::scrollable::RelativeOffset::START.into(),
                ));
            }
        }
        // ── 文件浏览器 ──
        Message::ToggleDir(path) => {
            if state.expanded_dirs.contains(&path) {
                state.expanded_dirs.remove(&path);
            } else {
                state.expanded_dirs.insert(path);
            }
        }
        Message::SelectFile(path) => {
            state.selected_file = Some(path);
        }
        // ── 看板 ──
        Message::MoveCard(card_id, target_col) => {
            if let Some(card) = find_card(&mut state.kanban_columns, &card_id) {
                let status = match target_col.as_str() {
                    "todo" => CardStatus::Todo,
                    "doing" => CardStatus::Doing,
                    "review" => CardStatus::Review,
                    "done" => CardStatus::Done,
                    _ => card.status.clone(),
                };
                let mut card = card;
                card.status = status;
                if let Some(col) = state.kanban_columns.iter_mut().find(|c| c.id == target_col) {
                    col.cards.push(card);
                }
            }
        }
        // ── 设置开关 ──
        Message::ToggleSetting(key) => {
            if let Some(v) = state.settings.get_mut(&key) {
                *v = !*v;
            }
        }

        // ── 主题（改色 / 改外观 / 改字号 → 重算全局调色板）──
        Message::SetAccent(a) => {
            state.accent = a;
            theme::apply(&state.accent, state.appearance, state.font_scale);
        }
        Message::SetAppearance(ap) => {
            state.appearance = ap;
            theme::apply(&state.accent, state.appearance, state.font_scale);
        }
        Message::SetFontScale(fs)  => {
            state.font_scale = fs;
            theme::apply(&state.accent, state.appearance, state.font_scale);
        }
        Message::SetViewMode(vm) => {
            state.view_mode = vm;
        }
        Message::GridExpand(id) => {
            state.selected_profile = id;
            state.view_mode = ViewMode::Single;
        }
    }
    iced::Task::none()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let icon_bar = icon_bar::view(state);

    // 中间区域：左面板（可选）+ 主视图（单视图聊天 / 宫格）+ 右抽屉（可选）
    let mut children: Vec<Element<'_, Message>> = Vec::new();
    if let Some(panel) = state.active_panel {
        children.push(left_panel::view(state, panel));
    }
    let main_view: Element<'_, Message> = match state.view_mode {
        ViewMode::Single => chat_area::view(state),
        ViewMode::Grid => grid_view::view(state),
    };
    children.push(main_view);
    if state.right_open {
        children.push(right_drawer::view(state));
    }

    let center = row(children).spacing(theme::CARD_GAP);

    let base = container(row![icon_bar, center].spacing(theme::CARD_GAP))
        .width(Length::Fill)
        .height(Length::Fill)
        // 1+3 布局：暗背板为底，三张卡片（左栏卡 / 聊天卡 / 右抽屉卡）浮在其上。
        // 顶部不留白——卡片直接顶到标题栏底部，与背板融为一体；
        // 左右与底部保留 CARD_GAP 间距，保留卡片浮起的层次感。
        .padding(Padding {
            top: 0.0,
            right: theme::CARD_GAP,
            bottom: theme::CARD_GAP,
            left: theme::CARD_GAP,
        })
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::bg_backboard())),
            ..Default::default()
        })
        .into();

    // 叠加层：模态弹窗（暗化背景 + 居中卡片）、新建弹窗，依次叠在最上层
    let mut layers: Vec<Element<'_, Message>> = vec![base];
    if let Some(o) = state.overlay {
        layers.push(overlay::view(state, o));
    }
    if let Some(d) = state.create_dialog {
        layers.push(agents_panel::create_dialog_view(state, d));
    }
    stack(layers).into()
}

// ── 单元测试：验证“打开弹窗 → 输入 → 确认创建”确实把卡片写入 State ──
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_create_agent_adds_card() {
        let (mut state, _) = new();
        let before = state.profiles.len();
        let _ = update(&mut state, Message::OpenCreateDialog(CreateDialog::Agent));
        let _ = update(&mut state, Message::CreateInputChanged("mytest".into()));
        let _ = update(&mut state, Message::ConfirmCreate);
        assert_eq!(state.profiles.len(), before + 1);
        // 新建 Agent 置顶（index 0），确保立即可见、不被沉到列表底部
        assert_eq!(state.profiles.first().unwrap().id, "mytest");
        // 新建后默认选中该卡片
        assert_eq!(state.selected_profile, "mytest");
        assert!(state.create_dialog.is_none(), "弹窗应在创建后关闭");
    }

    #[test]
    fn confirm_create_project_adds_card() {
        let (mut state, _) = new();
        let before = state.projects.len();
        let _ = update(&mut state, Message::OpenCreateDialog(CreateDialog::Project));
        let _ = update(&mut state, Message::CreateInputChanged("projx".into()));
        let _ = update(&mut state, Message::ConfirmCreate);
        assert_eq!(state.projects.len(), before + 1);
        // 新项目紧随「主目录」桶（index 1），保持 Home 恒首且新项目立即可见
        assert_eq!(state.projects[1].id, "projx");
        assert_eq!(state.selected_project.as_deref(), Some("projx"));
    }

    #[test]
    fn confirm_create_empty_name_is_noop() {
        let (mut state, _) = new();
        let before = state.profiles.len();
        let _ = update(&mut state, Message::OpenCreateDialog(CreateDialog::Agent));
        let _ = update(&mut state, Message::CreateInputChanged("   ".into()));
        let _ = update(&mut state, Message::ConfirmCreate);
        assert_eq!(state.profiles.len(), before, "空名称不应创建卡片");
    }

    // 渲染回归：创建卡片后整体 view / 弹窗 view 不得 panic（否则卡片会“看不到”）
    #[test]
    fn view_renders_after_create_no_panic() {
        let (mut state, _) = new();

        // Agent：打开 → 输入 → 确认 → 渲染
        let before_p = state.profiles.len();
        let _ = update(&mut state, Message::OpenCreateDialog(CreateDialog::Agent));
        let _ = agents_panel::view(&state); // 弹窗渲染
        let _ = update(&mut state, Message::CreateInputChanged("testagent".into()));
        let _ = update(&mut state, Message::ConfirmCreate);
        assert_eq!(state.profiles.len(), before_p + 1);
        let _ = view(&state); // 含新卡片的整体渲染

        // Project：同理
        let before_j = state.projects.len();
        let _ = update(&mut state, Message::OpenCreateDialog(CreateDialog::Project));
        let _ = update(&mut state, Message::CreateInputChanged("testproj".into()));
        let _ = update(&mut state, Message::ConfirmCreate);
        assert_eq!(state.projects.len(), before_j + 1);
        let _ = view(&state);
    }
}
