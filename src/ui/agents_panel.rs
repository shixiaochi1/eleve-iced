// ============================================================
// AgentsPanel — 对齐 Tauri AgentsPanel.tsx
//   上部：ProfilePanel（Agent 卡片列表）
//   分隔线
//   下部：ProjectTreePanel（项目总览 / 钻取）
//   叠加：新建 Agent / 新建项目 模态弹窗
//
// 设计要点（对齐 Tauri 细节，无 Rust 反模式）：
//   - 所有 view 函数纯依赖 `&State`，返回 `Element<'a, Message>`
//   - 选中态：左侧竖条 + 浅强调色底 + 30% 强调色描边 + 发光阴影环
//   - 卡片整体不是 button（避免 button 嵌套 button），主体选择区是
//     button，删除/钻取/更多按钮是兄弟节点，互不嵌套
// ============================================================

use std::path::PathBuf;

use iced::widget::{
    button, column, container, row, rule, scrollable, text, text_input, MouseArea, Space, Svg,
};
use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Shadow, Vector,
};

use crate::ui::{
    AgentProfile, CreateDialog, LaneGroup, Message, ProjectNode, RepoNode, SessionPreview, State,
    theme,
};

// ───────────────────────────────────────────────────────────
// 入口：AgentsPanel 整体（单卡片：上 Agent / 分隔 / 下 Project）
// ───────────────────────────────────────────────────────────

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let content = column![
        profiles_section(state),
        rule_h(),
        projects_section(state),
    ]
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card_style())
        .into()
}

// ───────────────────────────────────────────────────────────
// 图标 / 通用小组件
// ───────────────────────────────────────────────────────────

fn icon_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/icons")
        .join(format!("{name}.svg"))
}

fn svg_icon<'a>(name: &str, size: f32, color: Color) -> Element<'a, Message> {
    Svg::from_path(icon_path(name))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_: &iced::Theme, _: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(color),
        })
        .into()
}

fn rule_h<'a>() -> Element<'a, Message> {
    rule::horizontal(1.0)
        .style(|_: &iced::Theme| rule::Style {
            color: theme::SEPARATOR,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
        .into()
}

fn bold_font() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    }
}

/// 计数胶囊（AGENTS / PROJECTS 头部的数字、会话数等）
fn count_badge<'a>(n: usize, color: Color) -> Element<'a, Message> {
    container(text(format!("{n}")).size(11).color(color))
        .padding([1.0, 6.0])
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(color, 0.15))),
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 默认 Agent 徽标
fn default_badge<'a>() -> Element<'a, Message> {
    container(text("默认").size(10).color(theme::ACCENT))
        .padding([1.0, 6.0])
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(theme::ACCENT, 0.15))),
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 自动发现项目徽标
fn auto_badge<'a>() -> Element<'a, Message> {
    container(text("自动").size(10).color(theme::TEXT_MUTED))
        .padding([1.0, 6.0])
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(theme::TEXT_MUTED, 0.12))),
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 项目主题色圆点（不带图标时用作项目图标）
fn color_dot<'a>(color: Color) -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fixed(12.0))
        .height(Length::Fixed(12.0))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 6.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

// ───────────────────────────────────────────────────────────
// 区块标题（AGENTS / PROJECTS）：标签 + 计数 + 新建药丸
// ───────────────────────────────────────────────────────────

fn section_header<'a>(
    label: &'a str,
    count: usize,
    create: Option<CreateDialog>,
) -> Element<'a, Message> {
    let new_btn: Element<'a, Message> = match create {
        Some(kind) => button(
            row![
                svg_icon("plus", 14.0, theme::TEXT_ON_ACCENT),
                text("新建").size(12),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .style(move |_: &iced::Theme, s| theme::primary_pill(s, theme::ACCENT))
        .on_press(Message::OpenCreateDialog(kind))
        .into(),
        None => Space::new().into(),
    };

    row![
        text(label).size(11).color(theme::TEXT_MUTED),
        count_badge(count, theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        new_btn,
    ]
    .align_y(Alignment::Center)
    .padding(theme::pad(10.0, 12.0, 8.0, 12.0))
    .into()
}

// ───────────────────────────────────────────────────────────
// 上部：ProfilePanel
// ───────────────────────────────────────────────────────────

fn profiles_section<'a>(state: &'a State) -> Element<'a, Message> {
    let cards: Vec<Element<'a, Message>> =
        state.profiles.iter().map(|p| profile_card(state, p)).collect();

    column![
        section_header("AGENTS", state.profiles.len(), Some(CreateDialog::Agent)),
        column(cards).spacing(6).padding(theme::pad(2.0, 8.0, 10.0, 8.0)),
    ]
    .into()
}

fn profile_card<'a>(state: &'a State, p: &'a AgentProfile) -> Element<'a, Message> {
    let selected = state.selected_profile == p.id;
    let accent = theme::accent_of(&p.color);

    // 左侧强调竖条（选中时点亮，flush 贴左）
    let bar = container(Space::new())
        .width(Length::Fixed(3.0))
        .height(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: if selected {
                Some(Background::Color(accent))
            } else {
                None
            },
            ..Default::default()
        });

    // 头像：圆形 + 首字母 glyph（无图则用 glyph；avatar_key 预设库以 glyph 代替）
    let initial = p
        .display_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    let avatar: Element<'a, Message> =
        container(text(initial).size(15).color(theme::TEXT_ON_ACCENT).font(bold_font()))
            .width(Length::Fixed(36.0))
            .height(Length::Fixed(36.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(Background::Color(accent)),
                border: Border {
                    radius: 18.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            })
            .into();

    // 顶部行：display_name +（默认徽标）
    let mut top_items: Vec<Element<'a, Message>> = vec![
        text(&p.display_name)
            .size(13)
            .color(theme::TEXT_PRIMARY)
            .font(bold_font())
            .into(),
    ];
    if p.is_default {
        top_items.push(default_badge());
    }
    let top = row(top_items).spacing(6).align_y(Alignment::Center);

    // 第二行：id（muted）
    let id_line = text(format!("@{id}", id = p.id)).size(10).color(theme::TEXT_MUTED);

    // 第三行：model · provider · skills
    let mut meta_parts: Vec<String> = Vec::new();
    if let Some(m) = &p.model {
        meta_parts.push(m.clone());
    }
    if let Some(pr) = &p.provider {
        meta_parts.push(pr.clone());
    }
    meta_parts.push(format!("{} skills", p.skill_count));
    let meta = text(meta_parts.join("  ·  ")).size(11).color(theme::TEXT_MUTED);

    let content = column![top, id_line, meta].spacing(3);
    let inner = row![avatar, content].spacing(10).align_y(Alignment::Center);

    // 主体选择区（button；hover 浅底反馈）
    let sel = button(inner)
        .width(Length::Fill)
        .padding(theme::pad(8.0, 10.0, 8.0, 10.0))
        .style(|_: &iced::Theme, _s| sel_btn_style(_s))
        .on_press(Message::SelectProfile(p.id.clone()));

    let mut row_items: Vec<Element<'a, Message>> = vec![bar.into(), sel.into()];
    // 删除按钮（默认 Agent 不可删；hover 时背景高亮提示可交互）
    if !p.is_default {
        let del = button(svg_icon("trash-2", 15.0, theme::TEXT_MUTED))
            .style(icon_btn_style(theme::ACCENT))
            .on_press(Message::DeleteProfile(p.id.clone()));
        row_items.push(del.into());
    }

    container(row(row_items).spacing(0).align_y(Alignment::Center))
        .padding(theme::pad(0.0, 10.0, 0.0, 0.0))
        .style(profile_card_style(selected, accent))
        .into()
}

// ───────────────────────────────────────────────────────────
// 下部：ProjectTreePanel（总览 / 钻取）
// ───────────────────────────────────────────────────────────

fn projects_section<'a>(state: &'a State) -> Element<'a, Message> {
    let body: Element<'a, Message> = match &state.drill_project {
        Some(id) => drill_view(state, id),
        None => overview_view(state),
    };

    column![
        section_header("PROJECTS", state.projects.len(), Some(CreateDialog::Project)),
        body,
    ]
    .height(Length::Fill)
    .into()
}

fn overview_view<'a>(state: &'a State) -> Element<'a, Message> {
    let cards: Vec<Element<'a, Message>> =
        state.projects.iter().map(|p| project_card(state, p)).collect();

    scrollable(column(cards).spacing(6).padding(theme::pad(2.0, 8.0, 10.0, 8.0)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn project_card<'a>(state: &'a State, p: &'a ProjectNode) -> Element<'a, Message> {
    let selected = state.selected_project.as_deref() == Some(p.id.as_str());
    let accent = theme::accent_of(&p.color);
    let is_open = state.expanded_projects.contains(&p.id);
    let has_children = !p.preview_sessions.is_empty() || !p.repos.is_empty();
    let can_drill = !p.is_no_project && !p.repos.is_empty();

    // 展开箭头（仅在有可展开内容时显示）
    let chevron = if has_children {
        svg_icon(
            if is_open { "chevron-down" } else { "chevron-right" },
            14.0,
            theme::TEXT_MUTED,
        )
    } else {
        container(Space::new())
            .width(Length::Fixed(14.0))
            .into()
    };

    // 项目图标：自定义 icon 覆盖 > Home（主目录）> 主题色圆点 > 文件夹
    let proj_icon = if p.is_no_project {
        svg_icon("house", 16.0, accent)
    } else if let Some(ic) = &p.icon {
        svg_icon(ic, 16.0, theme::TEXT_MUTED)
    } else if p.color.is_some() {
        color_dot(accent)
    } else {
        svg_icon("folder-kanban", 16.0, theme::TEXT_MUTED)
    };

    // 标签 +（非主目录）路径副标题
    let mut label_col_items: Vec<Element<'a, Message>> =
        vec![text(&p.label).size(13).color(theme::TEXT_PRIMARY).into()];
    if !p.is_no_project {
        if let Some(path) = &p.path {
            label_col_items.push(text(path).size(10).color(theme::TEXT_MUTED).into());
        }
    }
    let label = column(label_col_items).spacing(2);

    // 右侧：会话数（非主目录）+ 钻取按钮 + 更多菜单
    let mut right_items: Vec<Element<'a, Message>> = Vec::new();
    if !p.is_no_project && p.session_count > 0 {
        right_items.push(
            text(format!("{}", p.session_count))
                .size(11)
                .color(theme::TEXT_MUTED)
                .into(),
        );
    }
    if can_drill {
        right_items.push(
            button(svg_icon("chevrons-right", 15.0, theme::TEXT_MUTED))
                .style(icon_btn_style(theme::ACCENT))
                .on_press(Message::EnterDrill(p.id.clone()))
                .into(),
        );
    }
    if !p.is_no_project {
        // 自动项目 = 忽略（dismiss），显式项目 = 删除
        right_items.push(
            button(svg_icon("ellipsis-vertical", 15.0, theme::TEXT_MUTED))
                .style(icon_btn_style(theme::ACCENT))
                .on_press(Message::DeleteProject(p.id.clone()))
                .into(),
        );
    }

    // 主行：[箭头, 图标, 标签(+路径), (自动徽标), 弹性空白, 右侧操作]
    let mut main_row_items: Vec<Element<'a, Message>> =
        vec![chevron, proj_icon, label.into()];
    if p.is_auto {
        main_row_items.push(auto_badge());
    }
    main_row_items.push(Space::new().width(Length::Fill).into());
    main_row_items.push(row(right_items).spacing(4).align_y(Alignment::Center).into());

    let main = button(row(main_row_items).spacing(8).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding(theme::pad(7.0, 8.0, 7.0, 8.0))
        .style(|_: &iced::Theme, s| sel_btn_style(s))
        .on_press(Message::SelectProject(p.id.clone()));

    let mut col_items: Vec<Element<'a, Message>> = vec![main.into()];
    if is_open && !p.preview_sessions.is_empty() {
        col_items.push(preview_sessions(p));
    }

    container(column(col_items).spacing(0))
        .padding(theme::pad(0.0, 8.0, 0.0, 8.0))
        .style(project_card_style(selected, accent))
        .into()
}

/// 展开态下的会话预览（Top N，对齐 Hermes PROJECT_PREVIEW_COUNT）
fn preview_sessions<'a>(p: &'a ProjectNode) -> Element<'a, Message> {
    let rows: Vec<Element<'a, Message>> = p
        .preview_sessions
        .iter()
        .map(|s| {
            row![
                Space::new().width(Length::Fixed(20.0)),
                svg_icon("message-square", 13.0, theme::TEXT_MUTED),
                text(&s.title).size(12).color(theme::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
                text(&s.last_active).size(10).color(theme::TEXT_MUTED),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .padding(theme::pad(3.0, 8.0, 3.0, 8.0))
            .into()
        })
        .collect();

    column(rows).into()
}

// ───────────────────────────────────────────────────────────
// 钻取视图：Repo → Lane → Session 树
// ───────────────────────────────────────────────────────────

fn drill_view<'a>(state: &'a State, project_id: &str) -> Element<'a, Message> {
    let project = match state.projects.iter().find(|p| p.id == project_id) {
        Some(p) => p,
        None => return column![].into(),
    };

    let back = button(
        row![
            svg_icon("arrow-left", 14.0, theme::TEXT_PRIMARY),
            text("项目").size(12).color(theme::TEXT_PRIMARY),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .style(|_: &iced::Theme, _: button::Status| button::Style {
        background: None,
        border: Border::default(),
        ..Default::default()
    })
    .on_press(Message::ExitDrill);

    let repos: Vec<Element<'a, Message>> = project.repos.iter().map(drill_repo).collect();

    let body = column![
        row![back, Space::new().width(Length::Fill)]
            .padding(theme::pad(2.0, 4.0, 6.0, 0.0)),
        rule_h(),
        column(repos).spacing(10).padding(theme::pad(6.0, 4.0, 10.0, 4.0)),
    ];

    scrollable(body).width(Length::Fill).height(Length::Fill).into()
}

fn drill_repo<'a>(repo: &'a RepoNode) -> Element<'a, Message> {
    let lanes: Vec<Element<'a, Message>> = repo.lanes.iter().map(drill_lane).collect();

    column![
        row![
            svg_icon("git-branch", 14.0, theme::TEXT_MUTED),
            text(&repo.label).size(12).color(theme::TEXT_PRIMARY).font(bold_font()),
            Space::new().width(Length::Fill),
            text(format!("{}", repo.session_count))
                .size(11)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .padding(theme::pad(2.0, 0.0, 2.0, 0.0)),
        column(lanes).spacing(4).padding(theme::pad(0.0, 0.0, 0.0, 16.0)),
    ]
    .spacing(4)
    .into()
}

fn drill_lane<'a>(lane: &'a LaneGroup) -> Element<'a, Message> {
    let sessions: Vec<Element<'a, Message>> = lane.sessions.iter().map(drill_session).collect();

    column![
        row![
            svg_icon("circle-dot", 13.0, theme::TEXT_MUTED),
            text(&lane.label).size(12).color(theme::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            text(format!("{}", lane.session_count))
                .size(11)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .padding(theme::pad(2.0, 0.0, 2.0, 0.0)),
        column(sessions)
            .spacing(2)
            .padding(theme::pad(0.0, 0.0, 0.0, 16.0)),
    ]
    .spacing(2)
    .into()
}

fn drill_session<'a>(s: &'a SessionPreview) -> Element<'a, Message> {
    row![
        svg_icon("message-square", 13.0, theme::TEXT_MUTED),
        text(&s.title).size(12).color(theme::TEXT_PRIMARY),
        Space::new().width(Length::Fill),
        text(&s.last_active).size(10).color(theme::TEXT_MUTED),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .padding(theme::pad(3.0, 6.0, 3.0, 0.0))
    .into()
}

// ───────────────────────────────────────────────────────────
// 新建弹窗（modal overlay）：新建 Agent / 新建项目
// ───────────────────────────────────────────────────────────

pub fn create_dialog_view<'a>(state: &'a State, kind: CreateDialog) -> Element<'a, Message> {
    let title = match kind {
        CreateDialog::Agent => "新建 Agent",
        CreateDialog::Project => "新建项目",
    };

    let input = text_input("请输入名称…", &state.create_input)
        .width(Length::Fill)
        .on_input(Message::CreateInputChanged)
        .on_submit(Message::ConfirmCreate);

    let card_content = column![
        text(title).size(16).color(theme::TEXT_PRIMARY).font(bold_font()),
        input,
        row![
            button(text("取消").size(13).color(theme::TEXT_PRIMARY))
                .style(|_: &iced::Theme, _: button::Status| button::Style {
                    background: Some(Background::Color(theme::BG_MUTED)),
                    border: Border::default(),
                    ..Default::default()
                })
                .on_press(Message::CloseCreateDialog),
            button(text("创建").size(13).color(theme::TEXT_ON_ACCENT))
                .style(|_: &iced::Theme, s| theme::primary_pill(s, theme::ACCENT))
                .on_press(Message::ConfirmCreate),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(14);

    // 卡片本身是一个 button（on_press=Dismiss 仅用于消费事件，避免点卡片关闭弹窗）
    let card = button(card_content)
        .style(|_: &iced::Theme, _: button::Status| button::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: theme::CARD_RADIUS.into(),
                width: 1.0,
                color: theme::SEPARATOR,
            },
            ..Default::default()
        })
        .on_press(Message::Dismiss);

    // 暗化背景全屏覆盖；点背景关闭，点卡片不关闭
    let dim = container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.5,
            })),
            ..Default::default()
        });

    MouseArea::new(dim)
        .on_press(Message::CloseCreateDialog)
        .into()
}

// ───────────────────────────────────────────────────────────
// 样式闭包
// ───────────────────────────────────────────────────────────

/// 卡片主体选择区按钮：透明底，hover/press 浅色反馈
fn sel_btn_style(status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => {
            Some(Background::Color(theme::BG_HOVER))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        border: Border::default(),
        ..Default::default()
    }
}

/// 图标按钮：透明底，hover/press 强调色浅底
fn icon_btn_style(hover: Color) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, s| {
        let (bg, col) = match s {
            button::Status::Hovered | button::Status::Pressed => (
                Some(Background::Color(theme::with_alpha(hover, 0.15))),
                hover,
            ),
            _ => (None, theme::TEXT_MUTED),
        };
        button::Style {
            background: bg,
            text_color: col,
            border: Border::default(),
            ..Default::default()
        }
    }
}

/// Agent 卡片容器：选中 = 浅强调色底 + 30% 强调色描边 + 发光阴影环
fn profile_card_style(
    selected: bool,
    accent: Color,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| {
        if selected {
            container::Style {
                background: Some(Background::Color(theme::with_alpha(accent, 0.10))),
                border: Border {
                    radius: 10.0.into(),
                    width: 1.0,
                    color: theme::with_alpha(accent, 0.30),
                },
                shadow: Shadow {
                    color: theme::with_alpha(accent, 0.22),
                    offset: Vector::new(0.0, 0.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            }
        } else {
            container::Style {
                background: None,
                border: Border {
                    radius: 10.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }
    }
}

/// 项目卡片容器：选中 = 浅强调色底 + 30% 强调色描边
fn project_card_style(
    selected: bool,
    accent: Color,
) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| {
        if selected {
            container::Style {
                background: Some(Background::Color(theme::with_alpha(accent, 0.10))),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: theme::with_alpha(accent, 0.30),
                },
                ..Default::default()
            }
        } else {
            container::Style::default()
        }
    }
}
