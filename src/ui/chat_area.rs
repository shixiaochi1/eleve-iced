// Chat area — 对齐 Tauri App.tsx / ToolStatusBar / ContextBar / InputArea
// 结构（从上到下）：
//   chat-card (rounded-12, bg-card, flex-col, flex:1)
//     ToolStatusBar (h-10, px-4, border-b) — Bot图标 + 状态文字
//     MessageContainer (flex-1) — 空态 / 消息列表
//     ContextBar — [+ 新建会话] [言格] [DeepSeek] [MoA] ··· [tokens] [百分比] [开始时间]
//                  进度条
//     InputArea (composer-surface) — 控制行：[≡] [+] [🎤] [🚫] [模型] [模式] [⚡] [🌐] ··· [发送]

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, Svg, rule, Id};
use iced::{Element, Length, Background, Border, Alignment, Color, Padding};
use std::path::PathBuf;

// Base path for assets (resolved at compile time so it works regardless of cwd)
const ASSET_BASE: &str = env!("CARGO_MANIFEST_DIR");

fn asset_path(name: &str) -> PathBuf {
    PathBuf::from(ASSET_BASE).join("assets/icons").join(format!("{}.svg", name))
}

use crate::ui::{ChatBlock, ChatMessage, Message, State, ToolStatus, theme, CHAT_SCROLL_ID};

// ============================================================
// View — 聊天区主入口
// ============================================================

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let tool_status = tool_status_bar_view();
    let messages = messages_view(state);
    let context_bar = context_bar_view();
    let input_area = input_area_view(state);

    let content = column![
        tool_status,     // h-10=40px
        messages,        // flex-1
        container(column![
            context_bar,
            progress_bar_view(),
            input_area,
        ].spacing(4))
            .width(Length::Fill)
            .padding(Padding::new(0.0).right(12.0).bottom(12.0).left(12.0)),
    ]
    .spacing(0);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: theme::CARD_RADIUS.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

// ============================================================
// ToolStatusBar — 对齐 Tauri ToolStatusBar.tsx
// ============================================================

fn tool_status_bar_view<'a>() -> Element<'a, Message> {
    let bot_icon = Svg::from_path(asset_path("bot"))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _| iced::widget::svg::Style {
            color: Some(Color::from_rgba(0.55, 0.55, 0.58, 0.4)),
        });

    let status_text = text("就绪")
        .size(12)
        .color(Color::from_rgba(0.55, 0.55, 0.58, 0.6));

    let left = row![bot_icon, status_text]
        .spacing(8)
        .align_y(Alignment::Center);

    let separator = rule::horizontal(1.0)
        .style(|_: &iced::Theme| rule::Style {
            color: theme::SEPARATOR,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        });

    column![
        container(row![left, Space::new().width(Length::Fill).height(Length::Shrink)]
            .spacing(8)
            .align_y(Alignment::Center))
        .width(Length::Fill)
        .height(Length::Fill),
        separator,
    ]
    .height(Length::Fixed(44.0))
    .padding(Padding::new(6.0).right(16.0).bottom(0.0).left(16.0))
    .spacing(0)
    .into()
}

// ============================================================
// Messages area — 对齐 Tauri MessageContainer
// ============================================================

fn messages_view<'a>(state: &'a State) -> Element<'a, Message> {
    if state.messages.is_empty() {
        return empty_state_view();
    }

    let messages: Vec<Element<'a, Message>> = state
        .messages
        .iter()
        .map(message_bubble)
        .collect();

    scrollable(column(messages).spacing(8))
        .id(Id::new(CHAT_SCROLL_ID))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn empty_state_view<'a>() -> Element<'a, Message> {
    let hint_text = row![
        text("Ctrl+N 新建会话").size(12).color(theme::TEXT_MUTED),
        text("Enter 发送").size(12).color(theme::TEXT_MUTED),
        text("Shift+Enter 换行").size(12).color(theme::TEXT_MUTED),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    let center = column![
        Svg::from_path(asset_path("Elogo"))
            .width(Length::Fixed(64.0))
            .height(Length::Fixed(64.0)),
        text("Eleve Agent").size(18).color(theme::TEXT_PRIMARY),
        text("你的 AI 智能助手 · 开始对话吧").size(13).color(theme::TEXT_MUTED),
        hint_text,
    ]
    .spacing(16)
    .align_x(Alignment::Center)
    .width(Length::Shrink);

    container(center)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .padding(Padding::new(64.0).right(24.0).bottom(32.0).left(24.0))
        .into()
}

// ============================================================
// ContextBar — 对齐 Tauri ContextBar.tsx
// ============================================================

fn context_bar_view<'a>() -> Element<'a, Message> {
    let new_session_btn = nav_btn("+", "新建会话");
    let mode_btn = nav_btn_small("言格");
    let deepseek_btn = deepseek_button();
    let moa_btn = moa_toggle();

    let left = row![new_session_btn, mode_btn, deepseek_btn, moa_btn]
        .spacing(4)
        .align_y(Alignment::Center);

    let info_right = row![
        text("30.2k / 1.0M tokens").size(11).color(theme::TEXT_MUTED),
        text("2.0%").size(11).color(theme::TEXT_MUTED),
        text("0秒前").size(11).color(theme::TEXT_MUTED),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let info_row = row![left, Space::new().width(Length::Fill).height(Length::Shrink), info_right]
        .align_y(Alignment::Center);

    column![info_row].into()
}

// ============================================================
// ProgressBar
// ============================================================

fn progress_bar_view<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fixed(40.0)).height(Length::Fixed(4.0)))
        .width(Length::Fill)
        .height(Length::Fixed(4.0))
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.22, 0.5))),
            border: Border { radius: 2.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

// ============================================================
// InputArea — 对齐 Tauri InputArea.tsx
// ============================================================

fn input_area_view<'a>(state: &'a State) -> Element<'a, Message> {
    let has_text = !state.input.trim().is_empty();

    let control_left = row![
        icon_btn("menu", "命令菜单"),
        icon_btn("plus", "附件"),
        icon_btn("mic", "语音输入"),
        icon_btn("ban", "静音"),
        model_pill("qwen3.7-plus"),
        mode_pill("标准"),
        icon_btn_small("zap", "快速模式"),
        icon_btn_small("globe", "联网搜索"),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    let send_btn = button(
        container(
            Svg::from_path(asset_path("arrow-up"))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .style(|_: &iced::Theme, _s: iced::widget::svg::Status| iced::widget::svg::Style {
                    color: Some(Color::from_rgb(0.08, 0.09, 0.10)),
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    )
    .width(Length::Fixed(30.0))
    .height(Length::Fixed(30.0))
    .padding(6)
    .style(move |_: &iced::Theme, status| {
        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        iced::widget::button::Style {
            background: if has_text {
                if hovered {
                    Some(Background::Color(Color::from_rgba(0.92, 0.93, 0.94, 0.85)))
                } else {
                    Some(Background::Color(theme::TEXT_PRIMARY))
                }
            } else {
                Some(Background::Color(Color::from_rgba(0.92, 0.93, 0.95, 0.30)))
            },
            border: Border { radius: 15.0.into(), ..Default::default() },
            ..Default::default()
        }
    })
    .on_press(Message::SendPressed);

    let control_row = row![
        control_left,
        Space::new().width(Length::Fill).height(Length::Shrink),
        send_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let textarea = text_input("向 Eleve 发送消息… (Enter 发送, / 命令)", &state.input)
        .padding(6)
        .size(14)
        .width(Length::Fill)
        .on_input(Message::InputChanged)
        .style(|_: &iced::Theme, _: iced::widget::text_input::Status| iced::widget::text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
            icon: theme::TEXT_MUTED,
            placeholder: theme::TEXT_MUTED,
            value: theme::TEXT_PRIMARY,
            selection: theme::ACCENT,
        });

    let inner = column![textarea, control_row]
        .spacing(4)
        .padding([5, 8]);

    container(inner)
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            border: Border {
                radius: 16.0.into(),
                width: 1.0,
                color: theme::SEPARATOR,
            },
            ..Default::default()
        })
        .into()
}

// ============================================================
// Helper: 导航按钮 (ContextBar)
// ============================================================

fn nav_btn<'a>(icon_text: &'static str, label: &'static str) -> Element<'a, Message> {
    let content = if icon_text == "+" {
        row![
            text("+").size(13).color(theme::TEXT_MUTED),
            text(label).size(12).color(theme::TEXT_MUTED),
        ]
        .spacing(4)
    } else {
        row![text(label).size(12).color(theme::TEXT_MUTED)]
    };

    button(content)
        .padding([4, 10])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if _hovered {
                    Some(Background::Color(theme::BG_HOVER))
                } else {
                    Some(Background::Color(theme::BG_CARD))
                },
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                },
                ..Default::default()
            }
        })
        .into()
}

fn nav_btn_small<'a>(label: &'static str) -> Element<'a, Message> {
    button(text(label).size(12).color(theme::TEXT_MUTED))
        .padding([4, 8])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if _hovered {
                    Some(Background::Color(theme::BG_HOVER))
                } else {
                    Some(Background::Color(theme::BG_CARD))
                },
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                },
                ..Default::default()
            }
        })
        .into()
}

fn deepseek_button<'a>() -> Element<'a, Message> {
    let bot_icon = Svg::from_path(asset_path("bot"))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _| iced::widget::svg::Style {
            color: Some(theme::TEXT_MUTED),
        });

    button(row![bot_icon, text("DeepSeek").size(12).color(theme::TEXT_MUTED)].spacing(4))
        .padding([4, 10])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            },
            ..Default::default()
        })
        .into()
}

fn moa_toggle<'a>() -> Element<'a, Message> {
    button(text("MoA").size(12).color(theme::TEXT_MUTED))
        .padding([4, 8])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            },
            ..Default::default()
        })
        .into()
}

// ============================================================
// Helper: 图标按钮 (InputArea 控制行)
// ============================================================

fn icon_btn<'a>(name: &'static str, _tooltip: &'static str) -> Element<'a, Message> {
    let icon = Svg::from_path(asset_path(name))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _s: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(theme::TEXT_MUTED),
        });

    button(icon)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .padding(0)
        .style(|_theme: &iced::Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered {
                    Some(Background::Color(theme::BG_HOVER))
                } else {
                    None
                },
                border: Border::default(),
                ..Default::default()
            }
        })
        .into()
}

fn icon_btn_small<'a>(name: &'static str, _tooltip: &'static str) -> Element<'a, Message> {
    let icon = Svg::from_path(asset_path(name))
        .width(Length::Fixed(12.0))
        .height(Length::Fixed(12.0))
        .style(|_: &iced::Theme, _s: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(theme::TEXT_MUTED),
        });

    button(icon)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .padding(0)
        .style(|_theme: &iced::Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered {
                    Some(Background::Color(theme::BG_HOVER))
                } else {
                    None
                },
                border: Border::default(),
                ..Default::default()
            }
        })
        .into()
}

fn model_pill<'a>(model: &'static str) -> Element<'a, Message> {
    button(text(model).size(11).color(theme::TEXT_MUTED))
        .padding([2, 6])
        .height(Length::Fixed(24.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            },
            ..Default::default()
        })
        .into()
}

fn mode_pill<'a>(mode: &'static str) -> Element<'a, Message> {
    button(text(mode).size(11).color(theme::TEXT_MUTED))
        .padding([2, 6])
        .height(Length::Fixed(24.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            },
            ..Default::default()
        })
        .into()
}

// ============================================================
// Message bubble
// ============================================================

fn message_bubble<'a>(msg: &'a ChatMessage) -> Element<'a, Message> {
    let is_user = msg.role == "user";
    let bg = if is_user {
        Color::from_rgb(0.23, 0.30, 0.45)
    } else {
        Color::from_rgb(0.19, 0.19, 0.21)
    };
    let alignment = if is_user {
        iced::alignment::Horizontal::Right
    } else {
        iced::alignment::Horizontal::Left
    };

    let block_elements: Vec<Element<'a, Message>> = msg
        .blocks
        .iter()
        .map(|block| render_block(block))
        .collect();

    let bubble = container(column(block_elements).spacing(8))
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

fn render_block<'a>(block: &'a ChatBlock) -> Element<'a, Message> {
    match block {
        ChatBlock::Text(t) => text(t).size(13).color(theme::TEXT_PRIMARY).into(),
        ChatBlock::Code { language, code } => code_block_view(language.as_deref(), code),
        ChatBlock::ToolCall { name, status, result } => tool_call_view(name, *status, result),
    }
}

fn code_block_view<'a>(language: Option<&'a str>, code: &'a str) -> Element<'a, Message> {
    let header = row![
        text(language.unwrap_or("text")).size(11).color(theme::TEXT_MUTED),
        Space::new().width(Length::Fill).height(Length::Shrink),
    ]
    .padding([6, 10])
    .align_y(Alignment::Center);

    let body = text(code)
        .size(12)
        .color(theme::TEXT_PRIMARY)
        .font(iced::Font::MONOSPACE);

    container(column![header, body].spacing(0))
        .width(Length::Fill)
        .padding(theme::pad(0.0, 0.0, 8.0, 0.0))
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgb(0.09, 0.10, 0.12))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            },
            ..Default::default()
        })
        .into()
}

fn tool_call_view<'a>(name: &'a str, status: ToolStatus, result: &'a str) -> Element<'a, Message> {
    let icon = Svg::from_path(asset_path("bot"))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _| iced::widget::svg::Style {
            color: Some(theme::TEXT_MUTED),
        });

    let status_color = status.accent();
    let status_pill = container(text(status.label()).size(10).color(status_color))
        .padding([2, 8])
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color {
                r: status_color.r,
                g: status_color.g,
                b: status_color.b,
                a: 0.15,
            })),
            border: Border { radius: 6.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        });

    let header = row![
        icon,
        text(name).size(13).color(theme::TEXT_PRIMARY).width(Length::Fill),
        status_pill,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let result_text = text(result).size(12).color(theme::TEXT_MUTED);

    container(column![header, result_text].spacing(6).padding(12))
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgb(0.11, 0.12, 0.14))),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            },
            ..Default::default()
        })
        .into()
}
