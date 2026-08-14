use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Length, Border, Background};

use crate::ui::{Message, NavSection};

// ============================================================
// Color constants
// ============================================================

const BG_CARD: Color = Color::from_rgb(0.14, 0.15, 0.17);
const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.93, 0.95);
const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.58);

const DRAWER_WIDTH: f32 = 280.0;
const CARD_RADIUS: f32 = 12.0;

// ============================================================
// View
// ============================================================

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
    };

    // Header with close button
    let header = row![
        text(section_name)
            .size(14)
            .color(TEXT_PRIMARY)
            .width(Length::Fill),
        button(text("✕").size(11))
            .padding([4, 8])
            .style(|_: &iced::Theme, _| iced::widget::button::Style::default())
            .on_press(Message::ToggleDrawer)
    ]
    .padding([12, 16])
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Mock content
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
                    text(*label).size(12).color(TEXT_MUTED).width(Length::Fill),
                    text(*value).size(12).color(TEXT_PRIMARY),
                ]
                .spacing(8),
            )
            .padding([8, 16])
            .into()
        })
        .collect();

    let content = scrollable(column(items).spacing(2)).height(Length::Fill);

    let drawer_content = column![header, content].spacing(0);

    // 抽屉卡片：圆角 + 背景色
    container(drawer_content)
        .width(Length::Fixed(DRAWER_WIDTH))
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(BG_CARD)),
            border: Border {
                radius: CARD_RADIUS.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}
