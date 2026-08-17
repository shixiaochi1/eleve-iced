mod agents_panel;
mod chat_area;
mod file_browser;
mod icon_bar;
mod kanban_panel;
mod left_panel;
mod overlay;
mod placeholder;
mod right_drawer;
mod theme;

use std::collections::{HashMap, HashSet};

use iced::widget::{container, row, stack};
use iced::{Element, Length, Padding, Background};

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

// ============================================================
// 数据模型（mock）
// ============================================================

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant"
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ProjectNode {
    pub name: String,
    pub path: String,
    pub children: Option<Vec<ProjectNode>>,
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

    // ── Agent 左侧栏 ──
    SelectProfile(String),
    ToggleProject(String),
    SelectProject(String),

    // ── 文件浏览器（右抽屉）──
    ToggleDir(String),
    SelectFile(String),

    // ── 看板（左面板）──
    MoveCard(String, String), // (card_id, target_column_id)

    // ── 设置开关（模态弹窗 mock）──
    ToggleSetting(String),
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

    // Agent 左侧栏
    pub profiles: Vec<(String, String)>, // (id, label)
    pub selected_profile: String,
    pub projects: Vec<ProjectNode>,
    pub expanded_projects: HashSet<String>,
    pub selected_project: Option<String>,

    // 文件浏览器
    pub fs_root_name: String,
    pub fs_nodes: Vec<FsNode>,
    pub expanded_dirs: HashSet<String>,
    pub selected_file: Option<String>,

    // 看板
    pub kanban_columns: Vec<KanbanColumn>,

    // 设置开关
    pub settings: HashMap<String, bool>,
}

fn initial_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: "user".into(),
            content: "你好，请介绍一下你自己".into(),
        },
        ChatMessage {
            role: "assistant".into(),
            content: "你好！我是 ELEVE Agent，一个基于 Rust 构建的 AI 智能体。我可以帮你写代码、分析数据、自动化任务等。有什么我可以帮你的吗？".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: "帮我把 eleve-iced 的布局按 Tauri 的方式重构一下".into(),
        },
        ChatMessage {
            role: "assistant".into(),
            content: "好的。Tauri 的设计是：主聊天区永远常驻，左侧图标栏的点击分为三类——打开左侧面板、切换右侧抽屉、弹出模态窗口。我现在就按这个模型来重构 iced 前端。".into(),
        },
    ]
}

fn initial_profiles() -> Vec<(String, String)> {
    vec![
        ("default".into(), "默认 Agent".into()),
        ("coder".into(), "代码助手".into()),
        ("analyst".into(), "数据分析师".into()),
        ("writer".into(), "文档写手".into()),
        ("ops".into(), "运维助手".into()),
    ]
}

fn initial_projects() -> Vec<ProjectNode> {
    use ProjectNode as P;
    vec![
        P {
            name: "eleve-core".into(),
            path: "/projects/eleve-core".into(),
            children: Some(vec![
                P { name: "src".into(), path: "/projects/eleve-core/src".into(), children: Some(vec![
                    P { name: "lib.rs".into(), path: "/projects/eleve-core/src/lib.rs".into(), children: None },
                    P { name: "app.rs".into(), path: "/projects/eleve-core/src/app.rs".into(), children: None },
                ]) },
                P { name: "Cargo.toml".into(), path: "/projects/eleve-core/Cargo.toml".into(), children: None },
            ]),
        },
        P {
            name: "eleve-app".into(),
            path: "/projects/eleve-app".into(),
            children: Some(vec![
                P { name: "session_service.rs".into(), path: "/projects/eleve-app/session_service.rs".into(), children: None },
                P { name: "profile_service.rs".into(), path: "/projects/eleve-app/profile_service.rs".into(), children: None },
            ]),
        },
        P {
            name: "eleve-gateway".into(),
            path: "/projects/eleve-gateway".into(),
            children: None,
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
    (
        State {
            active_panel: Some(LeftPanel::Agents),
            right_open: false,
            right_tab: RightTab::Files,
            overlay: None,
            input: String::new(),
            messages: initial_messages(),
            profiles: initial_profiles(),
            selected_profile: "default".into(),
            projects: initial_projects(),
            expanded_projects: {
                let mut s = HashSet::new();
                s.insert("/projects/eleve-core".to_string());
                s
            },
            selected_project: Some("/projects/eleve-core".into()),
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
        },
        iced::Task::none(),
    )
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
                state.messages.push(ChatMessage { role: "user".into(), content: text.clone() });
                let reply = format!("（演示）已收到你的消息：「{}」。这是一段模拟回复，后端接入后将由真实 Agent 生成。", text);
                state.messages.push(ChatMessage { role: "assistant".into(), content: reply });
                state.input.clear();
            }
        }
        // ── Agent 左侧栏 ──
        Message::SelectProfile(id) => {
            state.selected_profile = id;
        }
        Message::ToggleProject(path) => {
            if state.expanded_projects.contains(&path) {
                state.expanded_projects.remove(&path);
            } else {
                state.expanded_projects.insert(path);
            }
        }
        Message::SelectProject(path) => {
            state.selected_project = Some(path);
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
    }
    iced::Task::none()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let icon_bar = icon_bar::view(state);

    // 中间区域：左面板（可选）+ 聊天区（常驻）+ 右抽屉（可选）
    let mut children: Vec<Element<'_, Message>> = Vec::new();
    if let Some(panel) = state.active_panel {
        children.push(left_panel::view(state, panel));
    }
    children.push(chat_area::view(state));
    if state.right_open {
        children.push(right_drawer::view(state));
    }

    let center = row(children).spacing(theme::CARD_GAP);

    let base = container(row![icon_bar, center])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0).right(theme::CARD_GAP).bottom(theme::CARD_GAP))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::BG_BACKBOARD)),
            ..Default::default()
        })
        .into();

    // 模态弹窗叠加在最上层（暗化背景 + 居中卡片）
    if let Some(o) = state.overlay {
        stack![base, overlay::view(state, o)].into()
    } else {
        base
    }
}
