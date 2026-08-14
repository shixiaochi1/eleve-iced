use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::ui::{Message, NavSection, theme};

pub fn view<'a>(_active_section: &'a NavSection) -> Element<'a, Message> {
    let title_bar = row![
        text("新会话")
            .size(14)
            .color(theme::TEXT_PRIMARY)
            .width(Length::Fill),
    ]
    .padding([8, 12])
    .align_y(iced::Alignment::Center);

    let messages: Vec<Element<'a, Message>> = mock_messages()
        .iter()
        .map(message_bubble)
        .collect();

    let messages_scroll = scrollable(column(messages).spacing(8)).height(Length::Fill);

    let input_bar = row![
        text_input("输入消息...", "")
            .padding(8)
            .size(13)
            .width(Length::Fill),
        button(text("发送").size(12).color(iced::Color::WHITE))
            .padding([8, 16])
            .style(|_: &iced::Theme, _| iced::widget::button::Style::default()
                .with_background(iced::Background::Color(theme::ACCENT))),
    ]
    .spacing(8)
    .padding([8, 12]);

    let content = column![
        title_bar,
        messages_scroll,
        input_bar,
    ]
    .spacing(0);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::BG_CARD)),
            border: iced::Border {
                radius: theme::CARD_RADIUS.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

// ============================================================
// Mock data
// ============================================================

struct MockMessage {
    role: &'static str,
    content: &'static str,
}

fn mock_messages() -> Vec<MockMessage> {
    vec![
        MockMessage {
            role: "user",
            content: "你好，请介绍一下你自己",
        },
        MockMessage {
            role: "assistant",
            content: "你好！我是 ELEVE Agent，一个基于 Rust 构建的 AI 智能体。我可以帮你写代码、分析数据、自动化任务等。有什么我可以帮你的吗？",
        },
        MockMessage {
            role: "user",
            content: "写一个 Rust 的 hello world",
        },
        MockMessage {
            role: "assistant",
            content: "当然！这是一个简单的 Rust Hello World 程序：\n\nfn main() {\n    println!(\"Hello, world!\");\n}\n\n运行方式：cargo run",
        },
    ]
}

fn message_bubble<'a>(msg: &MockMessage) -> Element<'a, Message> {
    let is_user = msg.role == "user";
    let bg = if is_user {
        iced::Color::from_rgb(0.23, 0.30, 0.45)
    } else {
        iced::Color::from_rgb(0.19, 0.19, 0.21)
    };
    let alignment = if is_user {
        iced::alignment::Horizontal::Right
    } else {
        iced::alignment::Horizontal::Left
    };

    let bubble = container(text(msg.content).size(13).color(theme::TEXT_PRIMARY))
        .padding(12)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 12.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        });

    let aligned = container(bubble)
        .width(Length::Fill)
        .align_x(alignment);

    container(aligned)
        .width(Length::Fill)
        .padding([4, 0])
        .into()
}
