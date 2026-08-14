use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::ui::{Message, NavSection, theme};

pub fn view<'a>(open: bool, active_section: &'a NavSection) -> Element<'a, Message> {
    if !open {
        return container(row![])
            .width(Length::Fixed(0.0))
            .height(Length::Fill)
            .into();
    }

    let section_name = match active_section {
        NavSection::Files => "文件浏览器",
        NavSection::Debug => "调试面板",
        NavSection::Settings => "设置",
        NavSection::Agent => "Agent 状态",
        NavSection::Kanban => "看板",
        NavSection::Cron => "定时任务",
        NavSection::Tools => "工具",
        NavSection::Learn => "学习",
        NavSection::Channels => "频道",
        NavSection::Usage => "用量",
        NavSection::Theme => "主题",
        NavSection::About => "关于",
    };

    let header = row![
        text(section_name)
            .size(14)
            .color(theme::TEXT_PRIMARY)
            .width(Length::Fill),
        button(text("✕").size(11))
            .padding([4, 8])
            .style(|_: &iced::Theme, _| iced::widget::button::Style::default())
            .on_press(Message::ToggleDrawer)
    ]
    .padding([12, 16])
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mock_items = vec![
        ("文件", "src/main.rs"),
        ("文件", "src/ui/mod.rs"),
        ("文件", "Cargo.toml"),
        ("状态", "Running"),
        ("内存", "~160 MB"),
        ("会话", "1 active"),
    ];

    let items: Vec<Element<'_, Message>> = mock_items
        .iter()
        .map(|(label, value)| {
            container(
                row![
                    text(*label).size(12).color(theme::TEXT_MUTED).width(Length::Fill),
                    text(*value).size(12).color(theme::TEXT_PRIMARY),
                ]
                .spacing(8),
            )
            .padding([8, 16])
            .into()
        })
        .collect();

    let content = scrollable(column(items).spacing(2)).height(Length::Fill);
    let drawer_content = column![header, content].spacing(0);

    container(drawer_content)
        .width(Length::Fixed(theme::DRAWER_WIDTH))
        .height(Length::Fill)
        .style(theme::card_style())
        .into()
}
