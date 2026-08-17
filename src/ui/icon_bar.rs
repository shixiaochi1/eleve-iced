// 图标栏（左侧竖向导航，60px）—— 对齐 Tauri IconBar.tsx
// 点击行为严格分三类：
//   · Logo / Agent / 看板 / 定时 / 工具 / 学习 / 频道 / 用量 / 调试 → 左面板 (ToggleLeftPanel)
//   · 文件 → 右侧抽屉 (ToggleFiles)
//   · 设置 / 主题 / 关于 → 模态弹窗 (OpenOverlay)

use iced::widget::{button, column, container, rule, Svg, Space};
use iced::{Element, Background, Length, Border, Padding};

use crate::ui::{LeftPanel, Message, RightTab, State, theme};

const ASSET_BASE: &str = env!("CARGO_MANIFEST_DIR");

fn asset_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(ASSET_BASE).join("assets/icons").join(format!("{}.svg", name))
}

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let logo_active = state.active_panel == Some(LeftPanel::Gateway);
    let files_active = state.right_open && state.right_tab == RightTab::Files;

    let logo_btn = icon_button("Elogo", logo_active, Message::ToggleLeftPanel(LeftPanel::Gateway), "网关状态");

    let nav_items = column![
        icon_button("users", state.active_panel == Some(LeftPanel::Agents), Message::ToggleLeftPanel(LeftPanel::Agents), "Agent"),
        icon_button("folder-git-2", files_active, Message::ToggleFiles, "文件浏览器"),
        icon_button("layout-grid", state.active_panel == Some(LeftPanel::Kanban), Message::ToggleLeftPanel(LeftPanel::Kanban), "看板"),
        icon_button("clock", state.active_panel == Some(LeftPanel::Cron), Message::ToggleLeftPanel(LeftPanel::Cron), "定时任务"),
        icon_button("wrench", state.active_panel == Some(LeftPanel::Tools), Message::ToggleLeftPanel(LeftPanel::Tools), "工具"),
        icon_button("book-open", state.active_panel == Some(LeftPanel::Learning), Message::ToggleLeftPanel(LeftPanel::Learning), "学习"),
        icon_button("radio", state.active_panel == Some(LeftPanel::Channels), Message::ToggleLeftPanel(LeftPanel::Channels), "频道"),
        icon_button("chart-column", state.active_panel == Some(LeftPanel::Usage), Message::ToggleLeftPanel(LeftPanel::Usage), "用量分析"),
        icon_button("bug", state.active_panel == Some(LeftPanel::Debug), Message::ToggleLeftPanel(LeftPanel::Debug), "调试"),
    ]
    .spacing(3)
    .align_x(iced::Alignment::Center);

    let bottom_items = column![
        icon_button("settings", false, Message::OpenOverlay(crate::ui::Overlay::Settings), "设置"),
        icon_button("palette", false, Message::OpenOverlay(crate::ui::Overlay::Theme), "主题"),
        icon_button("info", false, Message::OpenOverlay(crate::ui::Overlay::About), "关于"),
    ]
    .spacing(2)
    .align_x(iced::Alignment::Center);

    let content = column![
        logo_btn,
        Space::new().height(Length::Fixed(8.0)),
        nav_items,
        Space::new().height(Length::Fill),
        rule::horizontal(1.0).style(|_: &iced::Theme| rule::Style {
            color: theme::SEPARATOR,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        }),
        bottom_items,
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center);

    let icon_padding = (theme::ICON_BAR_WIDTH - theme::ICON_BTN_SIZE) / 2.0;
    container(content)
        .width(Length::Fixed(theme::ICON_BAR_WIDTH))
        .height(Length::Fill)
        .padding(Padding::new(0.0).bottom(theme::ICON_BAR_PADDING).left(icon_padding).right(icon_padding))
        .style(|_: &iced::Theme| container::Style { background: None, ..Default::default() })
        .into()
}

/// 通用图标按钮：is_active 时填充强调色渐变 + 指示条；hover 时浅色背景
fn icon_button<'a>(
    icon_name: &'static str,
    is_active: bool,
    on_press: Message,
    tooltip: &'static str,
) -> Element<'a, Message> {
    let icon_color = if is_active { theme::TEXT_ON_ACCENT } else { theme::TEXT_MUTED };

    let btn = button(
        Svg::from_path(asset_path(icon_name))
            .width(Length::Fixed(theme::ICON_SIZE))
            .height(Length::Fixed(theme::ICON_SIZE))
            .style(move |_: &iced::Theme, _| iced::widget::svg::Style { color: Some(icon_color) }),
    )
    .width(Length::Fixed(theme::ICON_BTN_SIZE))
    .height(Length::Fixed(theme::ICON_BTN_SIZE))
    .padding(theme::NAV_ICON_PADDING)
    .style(move |_: &iced::Theme, status| {
        let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
        if is_active {
            iced::widget::button::Style {
                background: Some(Background::Gradient(iced::Gradient::Linear(
                    iced::gradient::Linear::new(iced::Degrees(180.0))
                        .add_stop(0.0, theme::ACCENT)
                        .add_stop(1.0, theme::ACCENT_HOVER),
                ))),
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        } else if is_hovered {
            iced::widget::button::Style {
                background: Some(Background::Color(theme::BG_HOVER)),
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
    .on_press(on_press);

    // 激活态右侧指示条（对齐 Tauri indicator）
    let _ = tooltip;
    if is_active {
        column![
            btn,
            container(Space::new().width(Length::Fixed(3.0)).height(Length::Fixed(16.0)))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(Background::Color(theme::TEXT_ON_ACCENT)),
                    border: Border { radius: 2.0.into(), ..Default::default() },
                    ..Default::default()
                }),
        ]
        .align_x(iced::Alignment::Center)
        .into()
    } else {
        column![btn].align_x(iced::Alignment::Center).into()
    }
}
