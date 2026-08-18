// 左侧面板容器 —— 对齐 Tauri SidePanel.tsx
// 根据 active_panel 渲染不同分区内容；聊天区始终在右侧常驻。
// Agents / Kanban 用专属模块；其余分区（cron/tools/learning/channels/usage/debug/gateway）
// 用 placeholder 的 section_view。

use iced::{Element, Length};

use crate::ui::{agents_panel, kanban_panel, placeholder, LeftPanel, Message, State, theme};

pub fn view<'a>(state: &'a State, panel: LeftPanel) -> Element<'a, Message> {
    let inner: Element<'a, Message> = match panel {
        LeftPanel::Agents => agents_panel::view(state),
        LeftPanel::Kanban => kanban_panel::view(state),
        other => placeholder::section_view(state, other),
    };

    iced::widget::container(inner)
        .width(Length::Fixed(theme::LEFT_PANEL_WIDTH))
        .height(Length::Fill)
        .style(theme::card_style()) // 左栏整体 = 一只浮在背板上的圆角卡片（对齐 Tauri .side-panel-card，1+3 布局的卡片1）
        .into()
}
