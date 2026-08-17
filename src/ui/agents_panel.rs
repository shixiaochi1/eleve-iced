// AgentsPanel — 对齐 Tauri AgentsPanel.tsx（Agent + 项目合并侧栏）
// 上部：Agent 卡片列表（可切换，高亮选中）
// 下部：项目树（可展开/折叠、可选中）

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Background, Border, Alignment, Color, Padding};

use crate::ui::{Message, ProjectNode, State, theme};

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let profiles = profiles_view(state);
    let projects = projects_view(state);

    let content = column![
        profiles,
        container(rule_h()),
        projects,
    ]
    .spacing(0);

    container(content)
        .width(Length::Fixed(theme::LEFT_PANEL_WIDTH))
        .height(Length::Fill)
        .style(theme::card_style())
        .into()
}

// ───────────────────────────────────────────────────────────
// 上部：Agent 卡片列表
// ───────────────────────────────────────────────────────────

fn section_header<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(12).color(theme::TEXT_MUTED))
        .padding([10, 14])
        .into()
}

fn profiles_view<'a>(state: &'a State) -> Element<'a, Message> {
    let header = section_header("Agent");

    let items: Vec<Element<'a, Message>> = state
        .profiles
        .iter()
        .map(|(id, label)| profile_row(id, label, state.selected_profile == *id))
        .collect();

    let list = column(items).spacing(2).padding(theme::pad(0.0, 8.0, 8.0, 8.0));

    column![header, list].into()
}

fn profile_row<'a>(id: &'a str, label: &'a str, selected: bool) -> Element<'a, Message> {
    let indicator = if selected {
        container(Space::new().width(Length::Fixed(3.0)).height(Length::Fill))
            .style(|_: &iced::Theme| container::Style {
                background: Some(Background::Color(theme::ACCENT)),
                ..Default::default()
            })
    } else {
        container(Space::new())
    };

    let body = row![
        indicator,
        column![
            text(label).size(13).color(theme::TEXT_PRIMARY),
            text(format!("profile://{}", id)).size(10).color(theme::TEXT_MUTED),
        ]
        .spacing(2)
        .padding([2, 0]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(body)
        .width(Length::Fill)
        .padding([8, 10])
        .style(move |_: &iced::Theme, _status| {
            let bg = if selected {
                Some(Background::Color(theme::BG_HOVER))
            } else {
                None
            };
            iced::widget::button::Style {
                background: bg,
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        })
        .on_press(Message::SelectProfile(id.to_string()))
        .into()
}

// ───────────────────────────────────────────────────────────
// 下部：项目树（递归展开）
// ───────────────────────────────────────────────────────────

fn projects_view<'a>(state: &'a State) -> Element<'a, Message> {
    let header = section_header("项目");

    let mut flat: Vec<(usize, &ProjectNode)> = Vec::new();
    fn walk<'b>(nodes: &'b [ProjectNode], depth: usize, out: &mut Vec<(usize, &'b ProjectNode)>, expanded: &std::collections::HashSet<String>) {
        for n in nodes {
            out.push((depth, n));
            if n.children.is_some() && expanded.contains(&n.path) {
                if let Some(kids) = &n.children {
                    walk(kids, depth + 1, out, expanded);
                }
            }
        }
    }
    walk(&state.projects, 0, &mut flat, &state.expanded_projects);

    let rows: Vec<Element<'a, Message>> = flat
        .iter()
        .map(|(depth, node)| project_row(node, *depth, state))
        .collect();

    let list = scrollable(column(rows).spacing(1).padding(theme::pad(0.0, 6.0, 8.0, 6.0)))
        .height(Length::Fill)
        .width(Length::Fill);

    column![header, list].into()
}

fn project_row<'a>(node: &'a ProjectNode, depth: usize, state: &'a State) -> Element<'a, Message> {
    let is_open = state.expanded_projects.contains(&node.path);
    let selected = state.selected_project.as_deref() == Some(node.path.as_str());
    let has_children = node.children.is_some();

    let chevron = if has_children {
        chevron_icon(is_open)
    } else {
        container(Space::new().width(Length::Fixed(12.0))).into()
    };

    let icon = folder_icon(has_children && is_open);

    let row_content = row![chevron, icon, text(&node.name).size(12).color(theme::TEXT_PRIMARY)]
        .spacing(6)
        .align_y(Alignment::Center);

    let padded = container(row_content)
        .padding([5, 8])
        .style(move |_: &iced::Theme| {
            let bg = if selected {
                Some(Background::Color(theme::BG_HOVER))
            } else {
                None
            };
            iced::widget::container::Style {
                background: bg,
                ..Default::default()
            }
        });

    let indent_pad = Padding::new(0.0).left(8.0 + depth as f32 * 12.0);

    let btn = button(padded)
        .width(Length::Fill)
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            border: Border::default(),
            ..Default::default()
        })
        .on_press(if has_children {
            Message::ToggleProject(node.path.clone())
        } else {
            Message::SelectProject(node.path.clone())
        });

    container(btn).padding(indent_pad).into()
}

// ───────────────────────────────────────────────────────────
// 小图标（用 lucide svg）
// ───────────────────────────────────────────────────────────

use std::path::PathBuf;
const ASSET_BASE: &str = env!("CARGO_MANIFEST_DIR");
fn icon(name: &str) -> PathBuf {
    PathBuf::from(ASSET_BASE).join("assets/icons").join(format!("{}.svg", name))
}

fn chevron_icon<'a>(open: bool) -> Element<'a, Message> {
    let name = if open { "chevron-down" } else { "chevron-right" };
    iced::widget::Svg::from_path(icon(name))
        .width(Length::Fixed(12.0))
        .height(Length::Fixed(12.0))
        .style(|_: &iced::Theme, _s: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(theme::TEXT_MUTED),
        })
        .into()
}

fn folder_icon<'a>(open: bool) -> Element<'a, Message> {
    let name = if open { "folder-open" } else { "folder" };
    iced::widget::Svg::from_path(icon(name))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _s: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(Color::from_rgb(0.85, 0.70, 0.30)),
        })
        .into()
}

fn rule_h<'a>() -> Element<'a, Message> {
    iced::widget::rule::horizontal(1.0)
        .style(|_: &iced::Theme| iced::widget::rule::Style {
            color: theme::SEPARATOR,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        })
        .into()
}
