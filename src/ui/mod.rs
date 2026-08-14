mod chat_area;
mod drawer;

use iced::widget::{button, column, container, row, text, rule};
use iced::{Element, Task, Color, Border, Background, Length, Gradient, alignment, Padding};

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(NavSection),
    ToggleDrawer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavSection {
    Agent, Files, Kanban, Cron, Tools, Learn, Channels, Usage, Debug, Settings,
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
const ICON_BAR_WIDTH: f32 = 52.0;
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

    let card1 = if *active_section != NavSection::Agent {
        Some(card_left_panel(active_section))
    } else { None };

    let card2 = chat_area::view(active_section);
    let card3 = drawer::view(drawer_open, active_section);

    // 工具按钮和卡片1紧贴，卡片之间间距8px
    let cards = if let Some(c1) = card1 {
        row![row![icon_bar, c1].spacing(0.0), card2, card3].spacing(CARD_GAP)
    } else {
        row![icon_bar, card2, card3].spacing(CARD_GAP)
    };

    container(cards)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0).right(CARD_GAP).bottom(CARD_GAP).left(0.0))
        .into()
}

// ============================================================
// 工具按钮列（嵌在窗体上，透明背景）
// ============================================================

fn icon_bar_column<'a>(active_section: &'a NavSection) -> Element<'a, Message> {
    let logo_btn = button(
        text("E").size(18).color(TEXT_PRIMARY)
            .width(Length::Fill).align_x(alignment::Horizontal::Center),
    )
    .width(Length::Fixed(ICON_BAR_WIDTH))
    .height(Length::Fixed(ICON_BTN_SIZE))
    .padding(0)
    .style(|_: &iced::Theme, _| iced::widget::button::Style::default())
    .on_press(Message::NavigateTo(NavSection::Agent));

    let nav_items: Vec<Element<'a, Message>> = vec![
        nav_icon("⚡", "Agent", NavSection::Agent, active_section),
        nav_icon("📁", "文件", NavSection::Files, active_section),
        nav_icon("📋", "看板", NavSection::Kanban, active_section),
        nav_icon("⏰", "定时", NavSection::Cron, active_section),
        nav_icon("🔧", "工具", NavSection::Tools, active_section),
        nav_icon("📚", "学习", NavSection::Learn, active_section),
        nav_icon("📡", "频道", NavSection::Channels, active_section),
        nav_icon("📊", "用量", NavSection::Usage, active_section),
        nav_icon("🐛", "调试", NavSection::Debug, active_section),
    ];

    let nav_list = column(nav_items).spacing(2);
    let bottom = column![rule::horizontal(1.0), nav_icon("⚙️", "设置", NavSection::Settings, active_section)].spacing(4);

    container(column![logo_btn, nav_list, bottom].spacing(4))
        .width(Length::Fixed(ICON_BAR_WIDTH))
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style { background: None, ..Default::default() })
        .into()
}

// ============================================================
// 导航图标按钮
// ============================================================

fn nav_icon<'a>(icon: &'static str, label: &'static str, section: NavSection, active_section: &'a NavSection) -> Element<'a, Message> {
    let is_active = section == *active_section;
    button(
        column![
            text(icon).size(18).color(if is_active { TEXT_ON_ACCENT } else { TEXT_MUTED }),
            text(label).size(8).color(if is_active { TEXT_ON_ACCENT } else { TEXT_MUTED })
                .width(Length::Fill).align_x(alignment::Horizontal::Center),
        ]
        .align_x(alignment::Horizontal::Center).spacing(1),
    )
    .width(Length::Fixed(ICON_BAR_WIDTH))
    .height(Length::Fixed(ICON_BTN_SIZE))
    .padding(4)
    .style(move |_: &iced::Theme, status| {
        let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
        if is_active {
            iced::widget::button::Style::default()
                .with_background(Background::Gradient(Gradient::Linear(
                    iced::gradient::Linear::new(iced::Degrees(180.0))
                        .add_stop(0.0, ACCENT).add_stop(1.0, ACCENT_HOVER),
                )))
        } else if is_hovered {
            iced::widget::button::Style::default().with_background(Background::Color(BG_HOVER))
        } else {
            iced::widget::button::Style::default()
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
