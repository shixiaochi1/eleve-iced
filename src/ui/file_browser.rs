// FileBrowserPanel — 对齐 Tauri FileBrowserPanel.tsx（简化 mock）
// 树状文件列表：目录展开/折叠、文件选中高亮、当前根目录标题、空/加载占位。

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Background, Border, Alignment, Color, Padding};

use crate::ui::{FsNode, Message, State, theme};

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let header = row![
        text("文件").size(14).color(theme::text_primary()).width(Length::Fill),
    ]
    .padding([12, 16])
    .spacing(8)
    .align_y(Alignment::Center);

    let root_label = container(
        row![
            folder_icon(false),
            text(&state.fs_root_name).size(11).color(theme::text_muted()),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([6, 16])
    .style(|_: &iced::Theme| container::Style {
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: theme::separator(),
        },
        ..Default::default()
    });

    let mut flat: Vec<(usize, &FsNode)> = Vec::new();
    fn walk<'b>(nodes: &'b [FsNode], depth: usize, out: &mut Vec<(usize, &'b FsNode)>, expanded: &std::collections::HashSet<String>) {
        for n in nodes {
            out.push((depth, n));
            if n.is_dir && expanded.contains(&n.path) {
                if let Some(kids) = &n.children {
                    walk(kids, depth + 1, out, expanded);
                }
            }
        }
    }
    walk(&state.fs_nodes, 0, &mut flat, &state.expanded_dirs);

    let rows: Vec<Element<'a, Message>> = flat
        .iter()
        .map(|(depth, node)| file_row(node, *depth, state))
        .collect();

    let list = if flat.is_empty() {
        empty_view()
    } else {
        scrollable(column(rows).spacing(1).padding(theme::pad(0.0, 6.0, 8.0, 6.0)))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    };

    let content = column![header, root_label, list].spacing(8);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn file_row<'a>(node: &'a FsNode, depth: usize, state: &'a State) -> Element<'a, Message> {
    let is_open = state.expanded_dirs.contains(&node.path);
    let selected = state.selected_file.as_deref() == Some(node.path.as_str());

    let chevron = if node.is_dir {
        chevron_icon(is_open)
    } else {
        container(Space::new().width(Length::Fixed(12.0))).into()
    };

    let icon = if node.is_dir {
        folder_icon(is_open)
    } else {
        file_icon()
    };

    let row_content = row![chevron, icon, text(&node.name).size(12).color(theme::text_primary())]
        .spacing(6)
        .align_y(Alignment::Center);

    let padded = container(row_content)
        .padding([5, 8])
        .style(move |_: &iced::Theme| {
            let bg = if selected {
                Some(Background::Color(theme::bg_hover()))
            } else {
                None
            };
            iced::widget::container::Style { background: bg, ..Default::default() }
        });

    let indent_pad = Padding::new(0.0).left(8.0 + depth as f32 * 12.0);

    let btn = button(padded)
        .width(Length::Fill)
        .style(|_: &iced::Theme, _status| iced::widget::button::Style {
            border: Border::default(),
            ..Default::default()
        })
        .on_press(if node.is_dir {
            Message::ToggleDir(node.path.clone())
        } else {
            Message::SelectFile(node.path.clone())
        });

    container(btn).padding(indent_pad).into()
}

fn empty_view<'a>() -> Element<'a, Message> {
    container(
        column![
            folder_icon(false),
            text("空目录").size(12).color(theme::text_muted()),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .into()
}

// ───────────────────────────────────────────────────────────
// 小图标
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
            color: Some(theme::text_muted()),
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

fn file_icon<'a>() -> Element<'a, Message> {
    iced::widget::Svg::from_path(icon("file"))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_: &iced::Theme, _s: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(Color::from_rgb(0.40, 0.62, 0.85)),
        })
        .into()
}
