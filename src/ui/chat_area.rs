// Chat area — 对齐 Tauri App.tsx / ToolStatusBar / ContextBar / InputArea
// 结构（从上到下）：
//   chat-card (rounded-12, bg-card, flex-col, flex:1)
//     ToolStatusBar (h-10, px-4, border-b) — Bot图标 + 状态文字
//     MessageContainer (flex-1) — 空态 / 消息列表
//     ContextBar — [+ 新建会话] [言格] [DeepSeek] [MoA] ··· [tokens] [百分比] [开始时间]
//                  进度条
//     InputArea (composer-surface) — 控制行：[≡] [+] [🎤] [🚫] [模型] [模式] [⚡] [🌐] ··· [发送]

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, Svg, rule, Id, rich_text};
use iced::widget::text::Span;
use iced::{Element, Length, Background, Border, Alignment, Color, Padding};
use std::path::PathBuf;

// Base path for assets (resolved at compile time so it works regardless of cwd)
const ASSET_BASE: &str = env!("CARGO_MANIFEST_DIR");

fn asset_path(name: &str) -> PathBuf {
    PathBuf::from(ASSET_BASE).join("assets/icons").join(format!("{}.svg", name))
}

use crate::ui::{ChatBlock, Message, State, ToolStatus, theme, CHAT_SCROLL_ID};

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
            background: Some(Background::Color(theme::bg_card())),
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
            color: theme::separator(),
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
        .enumerate()
        .map(|(i, _)| message_bubble(state, i))
        .collect();

    scrollable(column(messages).spacing(8))
        .id(Id::new(CHAT_SCROLL_ID))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn empty_state_view<'a>() -> Element<'a, Message> {
    let hint_text = row![
        text("Ctrl+N 新建会话").size(12).color(theme::text_muted()),
        text("Enter 发送").size(12).color(theme::text_muted()),
        text("Shift+Enter 换行").size(12).color(theme::text_muted()),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    let center = column![
        Svg::from_path(asset_path("Elogo"))
            .width(Length::Fixed(64.0))
            .height(Length::Fixed(64.0)),
        text("Eleve Agent").size(18).color(theme::text_primary()),
        text("你的 AI 智能助手 · 开始对话吧").size(13).color(theme::text_muted()),
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
        text("30.2k / 1.0M tokens").size(11).color(theme::text_muted()),
        text("2.0%").size(11).color(theme::text_muted()),
        text("0秒前").size(11).color(theme::text_muted()),
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
                    Some(Background::Color(theme::text_primary()))
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
            icon: theme::text_muted(),
            placeholder: theme::text_muted(),
            value: theme::text_primary(),
            selection: theme::accent(),
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
                color: theme::separator(),
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
            text("+").size(13).color(theme::text_muted()),
            text(label).size(12).color(theme::text_muted()),
        ]
        .spacing(4)
    } else {
        row![text(label).size(12).color(theme::text_muted())]
    };

    button(content)
        .padding([4, 10])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if _hovered {
                    Some(Background::Color(theme::bg_hover()))
                } else {
                    Some(Background::Color(theme::bg_card()))
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
    button(text(label).size(12).color(theme::text_muted()))
        .padding([4, 8])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if _hovered {
                    Some(Background::Color(theme::bg_hover()))
                } else {
                    Some(Background::Color(theme::bg_card()))
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
            color: Some(theme::text_muted()),
        });

    button(row![bot_icon, text("DeepSeek").size(12).color(theme::text_muted())].spacing(4))
        .padding([4, 10])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::bg_card())),
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
    button(text("MoA").size(12).color(theme::text_muted()))
        .padding([4, 8])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::bg_card())),
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
            color: Some(theme::text_muted()),
        });

    button(icon)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .padding(0)
        .style(|_theme: &iced::Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered {
                    Some(Background::Color(theme::bg_hover()))
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
            color: Some(theme::text_muted()),
        });

    button(icon)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .padding(0)
        .style(|_theme: &iced::Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered {
                    Some(Background::Color(theme::bg_hover()))
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
    button(text(model).size(11).color(theme::text_muted()))
        .padding([2, 6])
        .height(Length::Fixed(24.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::bg_card())),
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
    button(text(mode).size(11).color(theme::text_muted()))
        .padding([2, 6])
        .height(Length::Fixed(24.0))
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::bg_card())),
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

fn message_bubble<'a>(state: &'a State, msg_index: usize) -> Element<'a, Message> {
    let msg = &state.messages[msg_index];
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

    let collapsed = state.thinking_collapsed.contains(&msg_index);

    let block_elements: Vec<Element<'a, Message>> = msg
        .blocks
        .iter()
        .map(|block| render_block(block, msg_index, collapsed))
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

fn render_block<'a>(block: &'a ChatBlock, msg_index: usize, collapsed: bool) -> Element<'a, Message> {
    match block {
        ChatBlock::Text(t) => markdown_view(t),
        ChatBlock::Code { language, code } => code_block_view(language.as_deref(), code),
        ChatBlock::ToolCall { name, status, result } => tool_call_view(name, *status, result),
        ChatBlock::Thinking { summary, detail } => thinking_view(summary, detail, msg_index, collapsed),
    }
}

/// 构造一个「链接消息类型 = Message」的拥有式 Span（无实际链接值），
/// 以满足 rich_text 对 `Link: Clone` 的约束，同时保持 Link 类型一致。
fn span(content: String) -> Span<'static, Message> {
    Span::new(content).link_maybe(None::<Message>)
}

/// 用 iced rich_text 渲染「行内 Markdown」：标题 / 列表 / 粗体 / 斜体 / 行内代码 / 链接。
/// 全部使用拥有的 Span（字符串自持），不借用外部数据，可在每次重绘时安全调用。
fn markdown_view<'a>(content: &'a str) -> Element<'a, Message> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks: Vec<Element<'a, Message>> = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // 无序列表（- / *）
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let mut items: Vec<&str> = Vec::new();
            while i < lines.len() {
                let t = lines[i].trim_start();
                if t.starts_with("- ") || t.starts_with("* ") {
                    items.push(t[2..].trim());
                    i += 1;
                } else if lines[i].trim().is_empty() {
                    break;
                } else {
                    break;
                }
            }
            let list: Vec<Element<'a, Message>> = items
                .iter()
                .map(|it| {
                    let mut spans = vec![
                        span("•  ".to_string()).size(13).color(theme::text_primary()),
                    ];
                    spans.extend(parse_inline(*it, 13.0));
                    container(rich_text(spans)).into()
                })
                .collect();
            blocks.push(container(column(list).spacing(4)).into());
            continue;
        }

        // 标题
        if let Some(rest) = trimmed.strip_prefix("### ") {
            blocks.push(heading_span(rest, 13.0));
            i += 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            blocks.push(heading_span(rest, 15.0));
            i += 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            blocks.push(heading_span(rest, 17.0));
            i += 1;
            continue;
        }

        // 空行
        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        // 段落：聚合到下一个空行 / 标题 / 列表之前
        let mut para = String::new();
        while i < lines.len()
            && !lines[i].trim().is_empty()
            && !lines[i].trim_start().starts_with("- ")
            && !lines[i].trim_start().starts_with("* ")
            && !lines[i].trim_start().starts_with('#')
        {
            if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(lines[i].trim());
            i += 1;
        }
        blocks.push(container(rich_text(parse_inline(&para, 13.0))).into());
    }

    if blocks.is_empty() {
        blocks.push(text("").into());
    }

    column(blocks).spacing(6).width(Length::Fill).into()
}

/// 解析一段行内文本为拥有式 Span 列表，支持 **粗体** / *斜体* / `代码` / [链接](url)。
fn parse_inline<'a>(src: &str, size: f32) -> Vec<Span<'a, Message>> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut out: Vec<Span<'a, Message>> = Vec::new();
    let mut plain = String::new();
    let mut i = 0usize;

    let flush = |plain: &mut String, out: &mut Vec<Span<'a, Message>>| {
        if !plain.is_empty() {
            out.push(span(std::mem::take(plain)).size(size).color(theme::text_primary()));
        }
    };

    while i < n {
        // 行内代码 `code`
        if chars[i] == '`' {
            flush(&mut plain, &mut out);
            let mut j = i + 1;
            let mut code = String::new();
            while j < n && chars[j] != '`' {
                code.push(chars[j]);
                j += 1;
            }
            i = (j + 1).min(n);
            out.push(
                span(code)
                    .size(size)
                    .font(iced::Font::MONOSPACE)
                    .color(theme::text_primary()),
            );
            continue;
        }
        // 粗体 **text**
        if i + 1 < n && chars[i] == '*' && chars[i + 1] == '*' {
            flush(&mut plain, &mut out);
            let mut j = i + 2;
            let mut bold = String::new();
            while j + 1 < n && !(chars[j] == '*' && chars[j + 1] == '*') {
                bold.push(chars[j]);
                j += 1;
            }
            i = (j + 2).min(n);
            out.push(span(bold).size(size).font(bold_font()).color(theme::text_primary()));
            continue;
        }
        // 斜体 *text*
        if chars[i] == '*' {
            flush(&mut plain, &mut out);
            let mut j = i + 1;
            let mut ital = String::new();
            while j < n && chars[j] != '*' {
                ital.push(chars[j]);
                j += 1;
            }
            i = (j + 1).min(n);
            out.push(span(ital).size(size).color(theme::text_muted()));
            continue;
        }
        // 链接 [label](url)
        if chars[i] == '[' {
            let mut j = i + 1;
            let mut label = String::new();
            while j < n && chars[j] != ']' {
                label.push(chars[j]);
                j += 1;
            }
            if j < n && chars.get(j + 1) == Some(&'(') {
                let mut k = j + 2;
                let mut url = String::new();
                while k < n && chars[k] != ')' {
                    url.push(chars[k]);
                    k += 1;
                }
                if k < n {
                    flush(&mut plain, &mut out);
                    out.push(
                        span(label)
                            .size(size)
                            .color(theme::accent())
                            .underline(true)
                            .link(Message::LinkClicked(url)),
                    );
                    i = (k + 1).min(n);
                    continue;
                }
            }
        }
        plain.push(chars[i]);
        i += 1;
    }
    flush(&mut plain, &mut out);
    out
}

/// 粗体字体（iced 默认不含字重，需要显式指定 Weight::Bold）。
fn bold_font() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    }
}

/// 渲染一个标题行（粗体 + 主文字色）。
fn heading_span<'a>(text: &str, size: f32) -> Element<'a, Message> {
    container(
        rich_text(vec![span(text.to_string())
            .size(size)
            .font(bold_font())
            .color(theme::text_primary())]),
    )
    .into()
}

/// 可折叠的「思考过程」卡片：折叠态显示一句话摘要，展开态显示完整 Markdown 内容。
fn thinking_view<'a>(
    summary: &'a str,
    detail: &'a str,
    msg_index: usize,
    collapsed: bool,
) -> Element<'a, Message> {
    let chevron = text(if collapsed { "▸" } else { "▾" })
        .size(12)
        .color(theme::text_muted());

    let brain = Svg::from_path(asset_path("brain"))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _| iced::widget::svg::Style {
            color: Some(theme::semantic_orange()),
        });

    let label = text("思考过程").size(12).color(theme::text_muted());

    let header_right = if collapsed {
        container(text(summary).size(12).color(theme::text_muted()))
            .width(Length::Fill)
            .padding(theme::pad(0.0, 8.0, 0.0, 8.0))
    } else {
        container(Space::new().width(Length::Fill).height(Length::Shrink))
    };

    let header = button(
        row![chevron, brain, label, header_right]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([2, 4])
    .style(|_: &iced::Theme, _status| iced::widget::button::Style {
        background: None,
        border: Border::default(),
        ..Default::default()
    })
    .on_press(Message::ToggleThinking(msg_index));

    let body = if collapsed {
        column![]
    } else {
        column![markdown_view(detail)].spacing(0)
    };

    container(column![header, body].spacing(6))
        .width(Length::Fill)
        .padding(theme::pad(4.0, 10.0, 10.0, 10.0))
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgb(0.13, 0.12, 0.11))),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color::from_rgba(0.95, 0.62, 0.20, 0.35),
            },
            ..Default::default()
        })
        .into()
}

fn code_block_view<'a>(language: Option<&'a str>, code: &'a str) -> Element<'a, Message> {
    let header = row![
        text(language.unwrap_or("text")).size(11).color(theme::text_muted()),
        Space::new().width(Length::Fill).height(Length::Shrink),
    ]
    .padding([6, 10])
    .align_y(Alignment::Center);

    let body = text(code)
        .size(12)
        .color(theme::text_primary())
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
            color: Some(theme::text_muted()),
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
        text(name).size(13).color(theme::text_primary()).width(Length::Fill),
        status_pill,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let result_text = text(result).size(12).color(theme::text_muted());

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
