mod chat_area;
mod drawer;

use iced::widget::{button, column, container, row, text, rule, Svg, Space};
use iced::{Element, Task, Color, Border, Background, Length, Gradient, Padding};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(NavSection),
    ToggleDrawer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavSection {
    Agent, Files, Kanban, Cron, Tools, Learn, Channels, Usage, Debug, Settings, Theme, About,
}

pub struct State {
    pub active_section: NavSection,
    pub drawer_open: bool,
}

// ============================================================
// Color system
// ============================================================

const BG_BACKBOARD: Color = Color::from_rgb(0.08, 0.09, 0.10);
const BG_CARD: Color = Color::from_rgb(0.14, 0.15, 0.17);
const BG_HOVER: Color = Color::from_rgb(0.18, 0.19, 0.21);
const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.93, 0.95);
const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.58);
const TEXT_ON_ACCENT: Color = Color::WHITE;
const ACCENT: Color = Color::from_rgb(0.38, 0.53, 0.89);
const ACCENT_HOVER: Color = Color { r: 0.38, g: 0.53, b: 0.89, a: 0.7 };

const CARD_RADIUS: f32 = 12.0;
const CARD_GAP: f32 = 8.0;
const LEFT_PANEL_WIDTH: f32 = 260.0;
const ICON_BAR_WIDTH: f32 = 60.0;
const ICON_BTN_SIZE: f32 = 40.0;
const TITLEBAR_HEIGHT: f32 = 32.0;
const DRAWER_WIDTH: f32 = 280.0;

// ============================================================
// Iced Application functions
// ============================================================

pub fn new() -> (State, Task<Message>) {
    (State { active_section: NavSection::Agent, drawer_open: false }, Task::none())
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NavigateTo(section) => {
            state.active_section = section;
            state.drawer_open = matches!(section, NavSection::Files | NavSection::Debug | NavSection::Settings);
        }
        Message::ToggleDrawer => { state.drawer_open = !state.drawer_open; }
    }
    Task::none()
}

pub fn view(state: &State) -> Element<'_, Message> {
    // 内容区：3张卡片直接顶到标题栏
    let content = content_row_view(&state.active_section, state.drawer_open);

    // 整个窗口 = 背板
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(BG_BACKBOARD)),
            ..Default::default()
        })
        .into()
}

// ============================================================
// 内容行
// ============================================================

fn content_row_view<'a>(active_section: &'a NavSection, drawer_open: bool) -> Element<'a, Message> {
    let icon_bar = icon_bar_column(active_section);

    // 左卡片：收起时完全不存在（非 Agent 模式展开）
    // 关键：icon_bar 独立于卡片区域，不共享 spacing，确保宽度始终 52px
    let card1_opt: Option<Element<'a, Message>> = if *active_section != NavSection::Agent {
        Some(card_left_panel(active_section))
    } else {
        None
    };

    let card2 = chat_area::view(active_section);
    let card3 = drawer::view(drawer_open, active_section);

    // 卡片区域：card1/card2/card3 之间间距 8px
    // icon_bar 独立在卡片区域外，和卡片区域无间距（紧贴）
    let cards_row = if let Some(c1) = card1_opt {
        row![c1, card2, card3].spacing(CARD_GAP)
    } else {
        row![card2, card3].spacing(CARD_GAP)
    };

    row![icon_bar, cards_row]
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0).top(CARD_GAP).right(CARD_GAP).bottom(CARD_GAP))
        .into()
}

// ============================================================
// 工具按钮列（嵌在窗体上，透明背景）
// ============================================================

fn icon_path(name: &str) -> PathBuf {
    PathBuf::from("assets/icons").join(format!("{}.svg", name))
}

fn icon_bar_column<'a>(active_section: &'a NavSection) -> Element<'a, Message> {
    // Logo 按钮 — 使用 Elogo.svg
    // Tauri: img w-6 h-6 (24px), button w-10 h-10 (40px)
    let logo_btn = button(
        Svg::from_path(PathBuf::from("assets/icons/Elogo.svg"))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0)),
    )
    .width(Length::Fixed(ICON_BTN_SIZE))
    .height(Length::Fixed(ICON_BTN_SIZE))
    .padding(0)
    .style(|_theme: &iced::Theme, status| {
        let is_active = matches!(status, iced::widget::button::Status::Hovered);
        iced::widget::button::Style {
            background: if is_active {
                Some(Background::Color(BG_HOVER))
            } else {
                None
            },
            border: Border { radius: 10.0.into(), ..Default::default() },
            ..Default::default()
        }
    })
    .on_press(Message::NavigateTo(NavSection::Agent));

    let nav_items: Vec<Element<'a, Message>> = vec![
        nav_icon("folder-git-2", NavSection::Files, active_section),
        nav_icon("layout-grid", NavSection::Kanban, active_section),
        nav_icon("clock", NavSection::Cron, active_section),
        nav_icon("wrench", NavSection::Tools, active_section),
        nav_icon("book-open", NavSection::Learn, active_section),
        nav_icon("radio", NavSection::Channels, active_section),
        nav_icon("chart-column", NavSection::Usage, active_section),
        nav_icon("bug", NavSection::Debug, active_section),
    ];

    // Tauri: 导航区用 flex-1 撑满，底部按钮自然推到最下面
    // Iced: 用 vertical_space 撑开实现同样效果
    let nav_list = column(nav_items).spacing(2);

    let bottom_items = column![
        nav_icon("settings", NavSection::Settings, active_section),
        nav_icon("palette", NavSection::Theme, active_section),
        nav_icon("info", NavSection::About, active_section),
    ]
    .spacing(2);

    let content = column![
        logo_btn,
        nav_list,
        Space::new().height(Length::Fill),
        rule::horizontal(1.0).style(|_: &iced::Theme| rule::Style {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        }),
        bottom_items,
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center);

    let icon_padding = (ICON_BAR_WIDTH - ICON_BTN_SIZE) / 2.0;
    container(content)
        .width(Length::Fixed(ICON_BAR_WIDTH))
        .height(Length::Fill)
        .padding(Padding::new(0.0).top(8.0).bottom(8.0).left(icon_padding).right(icon_padding))
        .style(|_: &iced::Theme| container::Style { background: None, ..Default::default() })
        .into()
}

// ============================================================
// 导航图标按钮（SVG 图标）
// ============================================================

fn nav_icon<'a>(icon_name: &'static str, section: NavSection, active_section: &'a NavSection) -> Element<'a, Message> {
    let is_active = section == *active_section;
    let icon_color = if is_active { TEXT_ON_ACCENT } else { TEXT_MUTED };

    button(
        Svg::from_path(icon_path(icon_name))
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .style(move |_: &iced::Theme, _| iced::widget::svg::Style { color: Some(icon_color) }),
    )
    .width(Length::Fixed(ICON_BTN_SIZE))
    .height(Length::Fixed(ICON_BTN_SIZE))
    .padding(10)  // (40-20)/2 = 10px 留白，图标居中
    .style(move |_: &iced::Theme, status| {
        let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
        if is_active {
            iced::widget::button::Style {
                background: Some(Background::Gradient(Gradient::Linear(
                    iced::gradient::Linear::new(iced::Degrees(180.0))
                        .add_stop(0.0, ACCENT).add_stop(1.0, ACCENT_HOVER),
                ))),
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        } else if is_hovered {
            iced::widget::button::Style {
                background: Some(Background::Color(BG_HOVER)),
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        } else {
            iced::widget::button::Style {
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        }
    })
    .on_press(Message::NavigateTo(section))
    .into()
}

// ============================================================
// 卡片1：左侧面板
// ============================================================

fn card_left_panel<'a>(active_section: &'a NavSection) -> Element<'a, Message> {
    let name = match active_section {
        NavSection::Files => "文件浏览器", NavSection::Kanban => "看板", NavSection::Cron => "定时任务",
        NavSection::Tools => "工具", NavSection::Learn => "学习", NavSection::Channels => "频道",
        NavSection::Usage => "用量分析", NavSection::Debug => "调试", NavSection::Settings => "设置",
        _ => "",
    };
    let header = container(text(name).size(14).color(TEXT_PRIMARY)).padding([12, 16]);
    let body = container(text("面板内容 - 待实现").size(12).color(TEXT_MUTED)).padding([4, 16]);
    container(column![header, body])
        .width(Length::Fixed(LEFT_PANEL_WIDTH))
        .height(Length::Fill)
        .style(card_container_style())
        .into()
}

fn card_container_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(Background::Color(BG_CARD)),
        border: Border { radius: CARD_RADIUS.into(), width: 0.0, color: Color::TRANSPARENT },
        ..Default::default()
    }
}
