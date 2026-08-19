// 宫格视图 — 对齐 Tauri GridModeView（多 Agent 卡片网格）
// 结构（从上到下）：
//   宫格卡片容器（圆角 12, bg-card, 占满主区域）
//     TopBar（h-9, border-b）— [单视图]  [N 个 Agent]  ···  拖拽/点击提示
//     Scrollable 网格 — responsive 按宽度 auto-fill 列数（MIN_CELL_W 下限），
//       每张卡片：色点头像 + 名称/默认徽标 + 模型/技能元信息 + 会话预览 + 展开提示
// 交互：点击卡片 = 选中该 Agent 并回到单视图（对齐「点击卡片聚焦 + 展开」）

use iced::widget::{button, column, container, responsive, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding};

use crate::ui::{AgentProfile, Message, State, ViewMode, theme};

const GAP: f32 = 10.0;
const PAD: f32 = 10.0;
const MIN_CELL_W: f32 = 320.0; // 卡片最小宽度（列数 = 容器宽度 auto-fill）

// ============================================================
// View — 宫格主入口
// ============================================================

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let count = state.profiles.len();

    let grid: Element<'a, Message> = responsive(move |size| {
        let w = size.width;
        let max_cols_by_width = ((w - PAD * 2.0 + GAP) / (MIN_CELL_W + GAP)).floor().max(1.0) as usize;
        let cols = max_cols_by_width.min(count.max(1));

        let rows: Vec<Element<'a, Message>> = state
            .profiles
            .chunks(cols)
            .map(|chunk| {
                let cards: Vec<Element<'a, Message>> =
                    chunk.iter().map(|p| agent_card(state, p)).collect();
                row(cards).spacing(GAP).width(Length::Fill).into()
            })
            .collect();

        column(rows).spacing(GAP).padding(PAD).into()
    })
    .into();

    let content = column![
        top_bar(count),
        scrollable(grid).width(Length::Fill).height(Length::Fill),
    ]
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
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
// TopBar — [单视图]  [N 个 Agent]  ···  提示
// ============================================================

fn top_bar(count: usize) -> Element<'static, Message> {
    let back_btn = button(
        row![
            text("◀").size(11).color(theme::text_muted()),
            text("单视图").size(11).color(theme::text_muted()),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding(Padding { top: 4.0, right: 8.0, bottom: 4.0, left: 8.0 })
    .style(|_t: &iced::Theme, status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(Background::Color(if hovered {
                theme::bg_hover()
            } else {
                theme::with_alpha(theme::bg_muted(), 0.5)
            })),
            border: Border { radius: 8.0.into(), ..Default::default() },
            text_color: theme::text_muted(),
            ..Default::default()
        }
    })
    .on_press(Message::SetViewMode(ViewMode::Single));

    let spacer: Element<'_, Message> = Space::new().width(Length::Fill).into();
    let hint: Element<'_, Message> = text(format!("{count} 个 Agent · 点击卡片展开进入单视图"))
        .size(10)
        .color(theme::with_alpha(theme::text_muted(), 0.5))
        .into();

    container(row![back_btn, spacer, hint].spacing(8).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding(Padding { top: 6.0, right: 12.0, bottom: 6.0, left: 12.0 })
        .style(|_: &iced::Theme| container::Style {
            background: None,
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

// ============================================================
// AgentCard — 单张宫格卡片（点击 = 选中 + 回单视图）
// ============================================================

fn agent_card<'a>(state: &'a State, p: &'a AgentProfile) -> Element<'a, Message> {
    let accent = theme::accent_of(&p.color);
    let selected = state.selected_profile == p.id;

    // 头像：色点 + 首字母 glyph（对齐 Tauri ProfileCard 头像位）
    let glyph = p.display_name.chars().next().unwrap_or('?');
    let avatar = container(text(glyph.to_string()).size(14).color(theme::text_on_accent()))
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(accent)),
            border: Border { radius: 9.0.into(), ..Default::default() },
            ..Default::default()
        });

    // 名称行：display_name (+ (id) 静音) + 默认徽标
    let mut name_items: Vec<Element<'a, Message>> = vec![
        text(&p.display_name)
            .size(14)
            .color(theme::text_primary())
            .into(),
    ];
    if p.display_name != p.id {
        name_items.push(
            text(format!("({})", p.id))
                .size(10)
                .color(theme::with_alpha(theme::text_muted(), 0.6))
                .into(),
        );
    }
    if p.is_default {
        name_items.push(default_badge());
    }
    let name_row = row(name_items).spacing(6.0).align_y(Alignment::Center);

    // 元信息行：模型 · 服务商 · 技能数
    let mut meta: Vec<String> = Vec::new();
    if let Some(m) = &p.model {
        meta.push(m.clone());
    }
    if let Some(pr) = &p.provider {
        meta.push(pr.clone());
    }
    meta.push(format!("{} 项技能", p.skill_count));
    let meta_text = text(meta.join(" · ")).size(10).color(theme::text_muted());

    // 会话预览（mock）：静态文案占位，真实后端接入后替换
    let preview = container(
        text("最近会话：你好，介绍一下你自己…")
            .size(11)
            .color(theme::text_muted()),
    )
    .width(Length::Fill)
    .padding(Padding::new(8.0))
    .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::bg_muted())),
        border: Border { radius: 8.0.into(), ..Default::default() },
        ..Default::default()
    });

    let expand_hint = row![
        text("点击展开").size(10).color(theme::accent()),
        text("→").size(10).color(theme::accent()),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let body = column![
        row![avatar, name_row].spacing(8.0).align_y(Alignment::Center),
        Space::new().height(Length::Fixed(6.0)),
        meta_text,
        Space::new().height(Length::Fixed(8.0)),
        preview,
        Space::new().height(Length::Fixed(8.0)),
        expand_hint,
    ]
    .spacing(0);

    button(body)
        .width(Length::Fill)
        .padding(Padding::new(12.0))
        .style(move |_: &iced::Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: Some(Background::Color(if hovered {
                    theme::bg_hover()
                } else {
                    theme::bg_elevated()
                })),
                border: Border {
                    radius: theme::CARD_RADIUS.into(),
                    width: 1.0,
                    color: if selected { theme::accent() } else { theme::separator() },
                },
                ..Default::default()
            }
        })
        .on_press(Message::GridExpand(p.id.clone()))
        .into()
}

fn default_badge<'a>() -> Element<'a, Message> {
    container(text("默认").size(9).color(theme::text_muted()))
        .padding(Padding { top: 1.0, right: 5.0, bottom: 1.0, left: 5.0 })
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(theme::bg_muted(), 0.8))),
            border: Border { radius: 5.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}
