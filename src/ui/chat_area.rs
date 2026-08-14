use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Border, Background};

use crate::ui::{Message, NavSection};

// ============================================================
// Color constants (Tauri版对齐)
// ============================================================

const BG_CARD: Color = Color::from_rgb(0.14, 0.15, 0.17); // 和左面板一样
const BG_USER_MSG: Color = Color::from_rgb(0.23, 0.30, 0.45);
const BG_AI_MSG: Color = Color::from_rgb(0.19, 0.19, 0.21);
const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.93, 0.95);
const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.58);
const BORDER: Color = Color::from_rgb(0.20, 0.21, 0.24);
const ACCENT: Color = Color::from_rgb(0.38, 0.53, 0.89);
const CARD_RADIUS: f32 = 12.0;

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

// ============================================================
// Message bubble
// ============================================================

fn message_bubble<'a>(msg: &MockMessage) -> Element<'a, Message> {
    let is_user = msg.role == "user";
    let bg = if is_user { BG_USER_MSG } else { BG_AI_MSG };
    let alignment = if is_user {
        iced::alignment::Horizontal::Right
    } else {
        iced::alignment::Horizontal::Left
    };

    let bubble = container(text(msg.content).size(13).color(TEXT_PRIMARY))
        .padding(12)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: 12.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
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

// ============================================================
// View
// ============================================================

pub fn view<'a>(_active_section: &'a NavSection) -> Element<'a, Message> {
    // 标题栏（和IconBar颜色一致 - transparent，嵌入背板）
    // 对齐Tauri .titlebar { background: transparent; }
    let title_bar = row![
        text("新会话")
            .size(14)
            .color(TEXT_PRIMARY)
            .width(Length::Fill),
        button(text("✕").size(11))
            .padding([4, 8])
            .style(|_: &iced::Theme, _| iced::widget::button::Style::default())
            .on_press(Message::ToggleDrawer)
    ]
    .padding([8, 12])
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // 消息列表
    let messages: Vec<Element<'a, Message>> = mock_messages()
        .iter()
        .map(message_bubble)
        .collect();

    let messages_scroll = scrollable(column(messages).spacing(8)).height(Length::Fill);

    // 输入区
    let input_bar = row![
        text_input("输入消息...", "")
            .padding(8)
            .size(13)
            .width(Length::Fill),
        button(text("发送").size(12).color(Color::WHITE))
            .padding([8, 16])
            .style(|_: &iced::Theme, _| iced::widget::button::Style::default()
                .with_background(Background::Color(ACCENT))),
    ]
    .spacing(8)
    .padding([8, 12]);

    // 主内容
    let content = column![
        title_bar,
        messages_scroll,
        input_bar,
    ]
    .spacing(0);

    // 聊天区卡片（和Tauri版 .chat-card 一样，有圆角和背景色）
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
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
