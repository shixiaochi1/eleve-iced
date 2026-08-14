// Chat area — 严格对齐 Tauri App.tsx L1420-1643 完整布局
// Tauri 结构（从上到下）：
//   chat-card (rounded-12, bg-card, flex-col, flex:1)
//     ToolStatusBar (h-10, px-4, border-b) — Bot图标 + 状态文字
//     MessageContainer (flex-1) — 空态 / 虚拟滚动消息
//     ContextBar — [+ 新建会话] [言格] [DeepSeek] [MoA] ··· [tokens] [百分比] [开始时间]
//                  进度条
//     InputArea (composer-surface) — 控制行：[≡] [+] [🎤] [🚫] [模型] [模式] [⚡] [🌐] ··· [发送]

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, Svg, rule};
use iced::{Element, Length, Background, Border, Alignment, Color, Padding};
use std::path::PathBuf;
// Base path for assets (resolved at compile time so it works regardless of cwd)
const ASSET_BASE: &str = env!("CARGO_MANIFEST_DIR");

fn asset_path(name: &str) -> PathBuf {
    PathBuf::from(ASSET_BASE).join("assets/icons").join(format!("{}.svg", name))
}

use crate::ui::{Message, NavSection, theme};

// ============================================================
// View — 聊天区主入口 (对应 Tauri chat-card)
// ============================================================

pub fn view<'a>(_active_section: &'a NavSection) -> Element<'a, Message> {
    let tool_status = tool_status_bar_view();
    let messages = messages_view();
    let context_bar = context_bar_view();
    let input_area = input_area_view();

    let content = column![
        tool_status,     // h-10=40px
        messages,        // flex-1
        context_bar,     // 按钮行全宽（无上下padding）
        // 进度条和输入框：共享左右下12px边距，上4px
        container(column![
            progress_bar_view(),
            input_area,
        ].spacing(4))
            .width(Length::Fill)
            .padding(Padding::new(4.0).right(12.0).bottom(12.0).left(12.0)),
    ]
    .spacing(0); // context_bar 到 container 无间距

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
// Tauri: flex items-center h-10 px-4 border-b border-border gap-2
// 左: Bot icon (14px) + 状态文字
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
        row![left, Space::new().width(Length::Fill).height(Length::Shrink)]
            .spacing(8)
            .align_y(Alignment::Center),
        separator,
    ]
    .height(Length::Fixed(40.0))
    .padding([0, 16])
    .into()
}

// ============================================================
// Messages area — 对齐 Tauri MessageContainer
// ============================================================

fn messages_view<'a>() -> Element<'a, Message> {
    let has_messages = false;

    if !has_messages {
        return empty_state_view();
    }

    let messages: Vec<Element<'a, Message>> = mock_messages()
        .iter()
        .map(message_bubble)
        .collect();

    scrollable(column(messages).spacing(8))
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
        // Tauri: w-16 h-16 (64px)
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
        .padding([32, 24])
        .into()
}

// ============================================================
// ContextBar — 对齐 Tauri ContextBar.tsx
// Tauri: flex items-center justify-between px-3 py-1.5
//   左: [+ 新建会话] [言格/模式切换] [DeepSeek] [MoA开关]
//   右: 30.2k / 1.0M tokens · 2.0% · 0秒前
//   进度条: h-1 mx-3 mt-0.5
// ============================================================

fn context_bar_view<'a>() -> Element<'a, Message> {
    // [+ 新建会话] — h-7=28px, px-2.5=10px, rounded-md
    let new_session_btn = nav_btn("+", "新建会话");

    // [言格] 模式切换按钮
    let mode_btn = nav_btn_small("言格");

    // [DeepSeek] 按钮
    let deepseek_btn = deepseek_button();

    // [MoA] 开关
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
        .padding([0, 12]) // 只保留左右12px，上下为0
        .align_y(Alignment::Center);

    column![info_row].into()
}

// ============================================================
// ProgressBar — 两端圆角，对齐输入框
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
// Tauri: composer-surface (rounded-2xl=16px, border)
//   textarea (bg-transparent, px-1, pt-1, pb-0.5, text-sm=14px)
//   控制行: gap-1=4px
//     左: [≡ 命令] [+] 附件 [🎤 语音] [🚫 静音] [模型下拉] [标准模式] [⚡ 快速] [🌐 联网]
//     右: [发送↑] (圆形, bg-primary, size-7.5=30px)
// CSS: pad-x=8px, pad-y=5px, row-gap=4px
// ============================================================

fn input_area_view<'a>() -> Element<'a, Message> {
    // 控制行 — 左侧按钮组 (对齐 Tauri size-(--composer-control-size)=28px, 但图标本身很小)
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

    // 发送按钮 — 圆形, 26px, 白色背景, 深色箭头 (Tauri size-(--composer-control-primary-size)=30px)
    let send_btn = button(
        Svg::from_path(asset_path("arrow-up"))
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0))
            .style(|_: &iced::Theme, _| iced::widget::svg::Style {
                color: Some(Color::from_rgb(0.08, 0.09, 0.10)),
            }),
    )
    .width(Length::Fixed(26.0))
    .height(Length::Fixed(26.0))
    .padding(0)
    .style(|_: &iced::Theme, _| iced::widget::button::Style {
        background: Some(Background::Color(theme::TEXT_PRIMARY)),
        border: Border { radius: 13.0.into(), ..Default::default() },
        ..Default::default()
    });

    let control_row = row![
        control_left,
        Space::new().width(Length::Fill).height(Length::Shrink),
        send_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    // 输入框 — 透明背景
    let textarea = text_input("向 Eleve 发送消息… (Enter 发送, / 命令)", "")
        .padding(6)
        .size(14)
        .width(Length::Fill)
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
        .padding([5, 8]); // pad-y=5px, pad-x=8px

    // composer-surface
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

// ============================================================
// Helper: DeepSeek 按钮
// ============================================================

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
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(theme::BG_CARD)),
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

// ============================================================
// Helper: MoA 开关
// ============================================================

fn moa_toggle<'a>() -> Element<'a, Message> {
    button(text("MoA").size(12).color(theme::TEXT_MUTED))
        .padding([4, 8])
        .height(Length::Fixed(28.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(theme::BG_CARD)),
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

// ============================================================
// Helper: 模型选择 pill
// ============================================================

fn model_pill<'a>(model: &'static str) -> Element<'a, Message> {
    button(text(model).size(11).color(theme::TEXT_MUTED))
        .padding([2, 6])
        .height(Length::Fixed(24.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(theme::BG_CARD)),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                },
                ..Default::default()
            }
        })
        .into()
}

// ============================================================
// Helper: 模式 pill
// ============================================================

fn mode_pill<'a>(mode: &'static str) -> Element<'a, Message> {
    button(text(mode).size(11).color(theme::TEXT_MUTED))
        .padding([2, 6])
        .height(Length::Fixed(24.0))
        .style(|_: &iced::Theme, status| {
            let _hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(theme::BG_CARD)),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                },
                ..Default::default()
            }
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
    ]
}

fn message_bubble<'a>(msg: &MockMessage) -> Element<'a, Message> {
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

    let bubble = container(text(msg.content).size(13).color(theme::TEXT_PRIMARY))
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
