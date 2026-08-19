// KanbanPanel — 对齐 Tauri KanbanPanel.tsx（简化 mock）
// 多列看板：列头（标题 + 计数），卡片（标题/状态标签/模型/Agent），
// 卡片可在相邻列间移动（← / → 按钮，纯前端 mock）。

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Background, Border, Alignment, Color};

use crate::ui::{KanbanCard, KanbanColumn, Message, State, theme};

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let header = row![
        text("看板").size(16).color(theme::text_primary()).width(Length::Fill),
        new_task_button(),
    ]
    .padding([14, 18])
    .spacing(8)
    .align_y(Alignment::Center);

    let cols: Vec<Element<'a, Message>> = state
        .kanban_columns
        .iter()
        .enumerate()
        .map(|(i, col)| column_view(col, i, state))
        .collect();

    let board = scrollable(row(cols).spacing(12).padding(theme::pad(0.0, 18.0, 18.0, 18.0)))
        .height(Length::Fill)
        .width(Length::Fill);

    container(column![header, board].spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card_style())
        .into()
}

fn new_task_button<'a>() -> Element<'a, Message> {
    button(
        row![
            text("+").size(13).color(theme::text_primary()),
            text("新建任务").size(12).color(theme::text_primary()),
        ]
        .spacing(4),
    )
    .padding([6, 12])
    .style(|_: &iced::Theme, _status| iced::widget::button::Style {
        background: Some(Background::Color(theme::accent())),
        border: Border { radius: 8.0.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}

fn column_view<'a>(col: &'a KanbanColumn, index: usize, state: &'a State) -> Element<'a, Message> {
    let count = col.cards.len();
    let header = row![
        status_dot(),
        text(&col.title).size(13).color(theme::text_primary()),
        container(text(format!("{}", count)).size(11).color(theme::text_muted()))
            .padding([1, 8])
            .style(|_: &iced::Theme| container::Style {
                background: Some(Background::Color(theme::separator())),
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }),
    ]
    .spacing(8)
    .padding([10, 12])
    .align_y(Alignment::Center);

    let cards: Vec<Element<'a, Message>> = col
        .cards
        .iter()
        .map(|card| card_view(card, index, state))
        .collect();

    let body = column(cards).spacing(8).padding(theme::pad(0.0, 8.0, 8.0, 8.0));

    container(column![header, body].spacing(0))
        .width(Length::Fixed(260.0))
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.11, 0.12, 0.14))),
            border: Border { radius: 10.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        })
        .into()
}

fn card_view<'a>(card: &'a KanbanCard, index: usize, state: &'a State) -> Element<'a, Message> {
    let last = state.kanban_columns.len() - 1;
    let left_target = if index > 0 { Some(state.kanban_columns[index - 1].id.clone()) } else { None };
    let right_target = if index < last { Some(state.kanban_columns[index + 1].id.clone()) } else { None };

    let title = text(&card.title).size(13).color(theme::text_primary());

    let tags = row![
        status_tag(card.status.label(), card.status.accent()),
        text(format!("@{}/{}", card.agent, card.model)).size(10).color(theme::text_muted()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let move_row = row![
        move_btn("←", Some(card.id.clone()), left_target),
        Space::new().width(Length::Fill),
        move_btn("→", Some(card.id.clone()), right_target),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    let inner = column![title, tags, move_row].spacing(8).padding(12);

    container(inner)
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::bg_card())),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: theme::separator(),
            },
            ..Default::default()
        })
        .into()
}

fn status_dot<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fixed(8.0)).height(Length::Fixed(8.0)))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::accent())),
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn status_tag<'a>(label: &'a str, color: Color) -> Element<'a, Message> {
    container(text(label).size(10).color(color))
        .padding([2, 8])
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 0.15,
            })),
            border: Border { radius: 6.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        })
        .into()
}

fn move_btn<'a>(label: &'static str, card_id: Option<String>, target: Option<String>) -> Element<'a, Message> {
    let enabled = card_id.is_some() && target.is_some();
    let content = text(label).size(11).color(if enabled { theme::text_muted() } else { theme::text_muted() });
    let btn = button(content)
        .padding([2, 8])
        .style(move |_: &iced::Theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(theme::separator())),
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        });
    if let (Some(cid), Some(tgt)) = (card_id, target) {
        btn.on_press(Message::MoveCard(cid, tgt)).into()
    } else {
        btn.into()
    }
}
