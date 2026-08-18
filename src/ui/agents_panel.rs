// ============================================================
// AgentsPanel — 对齐 Tauri AgentsPanel.tsx
//   上部：ProfilePanel（Agent 卡片列表）
//   分隔线
//   下部：ProjectTreePanel（项目总览 / 钻取）
//   叠加：新建 Agent / 新建项目 模态弹窗
//
// 视觉严格对齐 Tauri（2026-08 定稿）：
//   - 卡片：圆角 10 · 常驻 30% 强调色描边 · 浅卡片底 · 微投影（shadow-sm）
//   - 选中：10% 强调色淡底 + 45% 描边 + 重投影（--theme-shadow-color-heavy）
//            + 左侧发光竖条（inset 6px · 右端圆角 · glow）
//   - 头像：24px 圆角透明容器，无图时显示主题色 Bot 图标（老大 2026-08-12：去方块底）
//   - 元信息：Cpu/Plug/Package 小图标 + 文字（flex-nowrap 等高）
//   - 项目图标：24px 圆角 muted/40 芯片；彩色项目=色点，Home=house，其余=图标
//   - 新建按钮：渐变药丸（from-primary to-primary/90）+ 高光描边 + hover 抬升
//   架构：所有 view 纯依赖 &State；事件走 Message；卡片主体 button，操作按钮为兄弟节点
// ============================================================

use std::path::PathBuf;

use iced::border::Radius;
use iced::widget::{
    button, column, container, hover, row, rule, scrollable, stack, text, text_input, MouseArea,
    Space, Svg, Id,
};
use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Shadow, Vector,
};

use crate::ui::{
    AgentProfile, CreateDialog, LaneGroup, Message, ProjectNode, RepoNode, SessionPreview, State,
    theme,
};

const SEP_LIGHT: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.06); // 预览会话分隔线（border/40）

// ───────────────────────────────────────────────────────────
// 入口：AgentsPanel 整体（单卡片：上 Agent / 分隔 / 下 Project）
// ───────────────────────────────────────────────────────────

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    // 整体单一滚动：AGENT 区 + 分隔线 + 项目区 同处一个 scrollable，
    // 卡片很多时右侧只出现一个滚动条（避免两段各自滚动、割裂体验）。
    let content = column![
        profiles_section(state),
        rule_h(),
        projects_section(state),
    ]
    .spacing(0)
    .width(Length::Fill);

    let scroller = scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .id(Id::new(crate::ui::AGENTS_PANEL_SCROLL_ID)); // 新建卡片后 snap_to 到此 id 滚到顶部

    container(scroller)
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

fn bold_font() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    }
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

fn rule_light<'a>() -> Element<'a, Message> {
    rule::horizontal(1.0)
        .style(|_: &iced::Theme| rule::Style {
            color: SEP_LIGHT,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
        .into()
}

/// 小尺寸药丸（会话数 / 计数）：muted/50 底 + muted 字（对齐 Tauri bg-muted/50）
fn muted_chip(label: String) -> Element<'static, Message> {
    container(text(label).size(10).color(theme::TEXT_MUTED))
        .padding(theme::pad(1.0, 6.0, 1.0, 6.0))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(theme::BG_MUTED, 0.5))),
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 默认 Agent 徽标（中性灰底，含 Star 图标；对齐 Tauri bg-muted text-muted-foreground）
fn default_badge<'a>() -> Element<'a, Message> {
    let inner = row![
        svg_icon("star", 9.0, theme::TEXT_MUTED),
        text("默认").size(9).color(theme::TEXT_MUTED),
    ]
    .spacing(2.0)
    .align_y(Alignment::Center);

    container(inner)
        .padding(theme::pad(1.0, 6.0, 1.0, 6.0))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::BG_MUTED)),
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 自动发现项目徽标（muted/50 底）
fn auto_badge<'a>() -> Element<'a, Message> {
    container(text("自动").size(10).color(theme::TEXT_MUTED))
        .padding(theme::pad(1.0, 6.0, 1.0, 6.0))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(theme::BG_MUTED, 0.5))),
            border: Border {
                radius: 999.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 主题色圆点（项目图标 / 会话状态点）
fn color_dot<'a>(size: f32, color: Color) -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: (size / 2.0).into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// Agent 头像：24px 圆角透明容器 + 主题色 Bot 图标（无图时；对齐 Tauri 老大 2026-08-12 去方块底）
fn agent_avatar<'a>(color: Color) -> Element<'a, Message> {
    container(svg_icon("bot", 13.0, color))
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            border: Border {
                radius: 6.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

/// 元信息小图标 + 文字（Cpu/Plug/Package）
fn meta_chip(icon: &str, label: String) -> Element<'static, Message> {
    row![
        svg_icon(icon, 9.0, theme::TEXT_MUTED),
        text(label).size(10).color(theme::TEXT_MUTED),
    ]
    .spacing(3.0)
    .align_y(Alignment::Center)
    .into()
}

/// 项目前置图标芯片：24px 圆角 muted/40 容器；彩色项目=色点，Home=house，其余=图标
fn lead_icon_box<'a>(p: &'a ProjectNode) -> Element<'a, Message> {
    let accent = theme::accent_of(&p.color);
    let inner: Element<'a, Message> = if p.is_no_project {
        svg_icon("house", 13.0, theme::TEXT_MUTED)
    } else if p.color.is_some() && p.icon.is_none() {
        color_dot(12.0, accent)
    } else if let Some(ic) = &p.icon {
        svg_icon(ic, 13.0, accent)
    } else {
        svg_icon("folder-kanban", 13.0, theme::TEXT_MUTED)
    };

    container(inner)
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::with_alpha(theme::BG_MUTED, 0.5))),
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
// 区块标题（AGENTS / 项目）：标签 + 计数 + 新建药丸
// ───────────────────────────────────────────────────────────

fn section_header<'a>(
    label: &'a str,
    count: usize,
    create: Option<CreateDialog>,
) -> Element<'a, Message> {
    let new_label = match create {
        Some(CreateDialog::Agent) => "新建 Agent",
        Some(CreateDialog::Project) => "新建项目",
        None => "",
    };
    let new_btn: Element<'a, Message> = match create {
        Some(kind) => button(
            row![
                svg_icon("plus", 12.0, theme::TEXT_ON_ACCENT),
                text(new_label).size(11),
            ]
            .spacing(4.0)
            .align_y(Alignment::Center),
        )
        .style(move |_: &iced::Theme, s| theme::primary_pill(s, theme::ACCENT))
        .on_press(Message::OpenCreateDialog(kind))
        .into(),
        None => Space::new().into(),
    };

    row![
        text(label).size(10).font(bold_font()).color(theme::with_alpha(theme::TEXT_MUTED, 0.6)),
        text(format!("{count}")).size(10).color(theme::with_alpha(theme::TEXT_MUTED, 0.4)),
        Space::new().width(Length::Fill),
        new_btn,
    ]
    .align_y(Alignment::Center)
    .spacing(4.0)
    .padding(theme::pad(10.0, 12.0, 8.0, 12.0))
    .into()
}

// ───────────────────────────────────────────────────────────
// 选中态：左侧发光竖条 / 卡片投影
// ───────────────────────────────────────────────────────────

/// 左侧发光竖条（仅选中渲染）：inset 6px · 右端圆角 · glow（对齐 Tauri 0 0 8px accent65%）
fn accent_bar<'a>(selected: bool, accent: Color) -> Element<'a, Message> {
    let inner: Element<'a, Message> = if selected {
        container(Space::new())
            .width(Length::Fixed(3.0))
            .height(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(Background::Color(accent)),
                border: Border {
                    radius: Radius::default().top_right(999.0).bottom_right(999.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow {
                    color: theme::with_alpha(accent, 0.65),
                    offset: Vector::new(0.0, 0.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            })
            .into()
    } else {
        container(Space::new())
            .width(Length::Fixed(3.0))
            .height(Length::Fill)
            .style(|_: &iced::Theme| container::Style::default())
            .into()
    };
    // inset 6px 顶/底
    container(inner)
        .padding(theme::pad(6.0, 0.0, 6.0, 0.0))
        .into()
}

/// 卡片投影：选中=重投影（0 6px 18px heavy，对齐 Tauri --theme-shadow-color-heavy）；未选中=微投影（shadow-sm）
fn card_shadow(selected: bool) -> Shadow {
    if selected {
        Shadow {
            color: theme::SHADOW_HEAVY,
            offset: Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        }
    } else {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 2.0,
        }
    }
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

    // mk_inner：构建卡片主体（头像 + 名称行 + 元信息行 + 右侧弹性留白）
    let mk_inner = move || -> Element<'a, Message> {
        // 头像：24px 圆角透明容器 + 主题色 Bot 图标
        let avatar = agent_avatar(accent);

        // 名称行：display_name (+ (id) 静音) + 默认徽标（仅默认 Agent 显示，数据驱动差异）
        let mut name_row_items: Vec<Element<'a, Message>> = vec![
            text(&p.display_name)
                .size(13)
                .font(bold_font())
                .color(theme::TEXT_PRIMARY)
                .into(),
        ];
        if p.display_name != p.id {
            name_row_items.push(
                text(format!("({})", p.id))
                    .size(10)
                    .color(theme::with_alpha(theme::TEXT_MUTED, 0.6))
                    .into(),
            );
        }
        if p.is_default {
            name_row_items.push(default_badge());
        }
        let name_row = row(name_row_items).spacing(6.0).align_y(Alignment::Center);

        // 元信息行：Cpu/Plug/Package（flex-nowrap 等高）
        let mut meta: Vec<Element<'a, Message>> = Vec::new();
        match &p.model {
            Some(m) => meta.push(meta_chip("cpu", m.clone())),
            None => meta.push(
                text("未配置模型")
                    .size(10)
                    .color(theme::with_alpha(theme::TEXT_MUTED, 0.4))
                    .into(),
            ),
        }
        if let Some(pr) = &p.provider {
            meta.push(meta_chip("plug", pr.clone()));
        }
        meta.push(meta_chip("package", format!("{}", p.skill_count)));
        let meta_row = row(meta).spacing(8.0).align_y(Alignment::Center);

        let body = column![name_row, meta_row].spacing(6.0);
        let header: Element<'a, Message> = row![avatar, body]
            .spacing(6.0)
            .align_y(Alignment::Center)
            .into();
        let spacer: Element<'a, Message> = Space::new().width(Length::Fill).into();
        let inner: Element<'a, Message> = row![header, spacer]
            .spacing(6.0)
            .align_y(Alignment::Center)
            .into();
        inner
    };

    // mk_actions：默认 Agent 不可删（数据驱动差异）；删除按钮仅在 hover 整卡时出现
    let mk_actions = move || -> Vec<Element<'a, Message>> {
        if !p.is_default {
            vec![delete_button(p.id.clone())]
        } else {
            vec![]
        }
    };

    card_frame(
        selected,
        accent,
        mk_inner,
        mk_actions,
        Message::SelectProfile(p.id.clone()),
    )
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
        section_header("项目", state.projects.len(), Some(CreateDialog::Project)),
        body,
    ]
    .height(Length::Fill)
    .into()
}

fn overview_view<'a>(state: &'a State) -> Element<'a, Message> {
    let cards: Vec<Element<'a, Message>> =
        state.projects.iter().map(|p| project_card(state, p)).collect();

    // 不再单独滚动：整体滚动由 AgentsPanel::view 的 scrollable 统一承担
    column(cards)
        .spacing(6)
        .padding(theme::pad(2.0, 8.0, 10.0, 8.0))
        .into()
}

fn project_card<'a>(state: &'a State, p: &'a ProjectNode) -> Element<'a, Message> {
    // 选中态 chrome 统一用主题 primary（对齐 Tauri：项目卡片描边/竖条/投影=主题色）
    let accent = theme::ACCENT;
    let selected = state.selected_project.as_deref() == Some(p.id.as_str());
    let is_open = state.expanded_projects.contains(&p.id);

    let mk_inner = move || -> Element<'a, Message> {
        // 展开箭头（常驻；hover 提亮；独立按钮，不嵌套在选中层内，避免事件被吞）
        let chevron = button(svg_icon(
            if is_open { "chevron-down" } else { "chevron-right" },
            14.0,
            theme::TEXT_MUTED,
        ))
        .style(|_: &iced::Theme, s| {
            let bg = match s {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(theme::with_alpha(theme::ACCENT, 0.30)))
                }
                _ => None,
            };
            button::Style {
                background: bg,
                border: Border::default(),
                ..Default::default()
            }
        })
        .on_press(Message::ToggleProjectExpand(p.id.clone()));

        let icon_box = lead_icon_box(p);

        // 名称行：箭头 + 图标芯片 + 标签 + (自动徽标) + (非 Home) 计数 + (非 Home) 时间
        let mut name_items: Vec<Element<'a, Message>> = vec![
            chevron.into(),
            icon_box,
            text(&p.label)
                .size(13)
                .font(bold_font())
                .color(theme::TEXT_PRIMARY)
                .into(),
        ];
        if !p.is_no_project {
            if p.is_auto {
                name_items.push(auto_badge());
            }
            if p.session_count > 0 {
                name_items.push(muted_chip(format!("{}", p.session_count)));
            }
            name_items.push(
                text(&p.last_active)
                    .size(10)
                    .color(theme::with_alpha(theme::TEXT_MUTED, 0.5))
                    .into(),
            );
        }
        let name_row = row(name_items).spacing(6.0).align_y(Alignment::Center);

        let header_items: Vec<Element<'a, Message>> =
            vec![name_row.into(), Space::new().width(Length::Fill).into()];
        let header = row(header_items).spacing(6.0).align_y(Alignment::Center);

        // 主体：header (+ 展开态预览)
        let mut inner_items: Vec<Element<'a, Message>> = vec![header.into()];
        if is_open {
            inner_items.push(preview_block(p, state.active_session.as_deref()));
        }
        let inner = column(inner_items).spacing(0.0);
        inner.into()
    };

    // mk_actions：更多按钮（kebab）仅 hover 整卡时出现（Home 桶无菜单）
    let mk_actions = move || -> Vec<Element<'a, Message>> {
        if !p.is_no_project {
            vec![kebab_button(p.id.clone())]
        } else {
            vec![]
        }
    };

    card_frame(
        selected,
        accent,
        mk_inner,
        mk_actions,
        Message::SelectProject(p.id.clone()),
    )
}

/// 展开态下的会话预览（Top N，对齐 Hermes PROJECT_PREVIEW_COUNT）
fn preview_block<'a>(p: &'a ProjectNode, active_id: Option<&str>) -> Element<'a, Message> {
    let rows: Vec<Element<'a, Message>> = if p.preview_sessions.is_empty() {
        vec![text("暂无会话")
            .size(10)
            .color(theme::with_alpha(theme::TEXT_MUTED, 0.5))
            .into()]
    } else {
        p.preview_sessions
            .iter()
            .map(|s| preview_session_row(s, active_id == Some(s.id.as_str())))
            .collect()
    };

    column![rule_light(), column(rows).spacing(2.0)]
        .padding(theme::pad(4.0, 8.0, 2.0, 0.0))
        .into()
}

/// 预览会话行：状态点 + 消息图标 + 标题 + 时间（pl-6 缩进；对齐 Tauri SessionItem）
fn preview_session_row<'a>(s: &'a SessionPreview, active: bool) -> Element<'a, Message> {
    let title_color = if active {
        theme::ACCENT_ORANGE
    } else {
        theme::TEXT_PRIMARY
    };
    let icon_color = if active {
        theme::ACCENT_ORANGE
    } else {
        theme::TEXT_MUTED
    };
    let dot = color_dot(6.0, if active { theme::ACCENT_ORANGE } else { theme::TEXT_MUTED });

    row![
        dot,
        svg_icon("message-square", 12.0, icon_color),
        text(&s.title).size(12).color(title_color),
        Space::new().width(Length::Fill),
        text(&s.last_active).size(10).color(theme::TEXT_MUTED),
    ]
    .spacing(6.0)
    .align_y(Alignment::Center)
    .padding(theme::pad(3.0, 8.0, 3.0, 24.0))
    .into()
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
        .spacing(4.0)
        .align_y(Alignment::Center),
    )
    .style(|_: &iced::Theme, s| {
        let bg = match s {
            button::Status::Hovered | button::Status::Pressed => {
                Some(Background::Color(theme::with_alpha(theme::ACCENT, 0.30)))
            }
            _ => None,
        };
        button::Style {
            background: bg,
            border: Border::default(),
            ..Default::default()
        }
    })
    .on_press(Message::ExitDrill);

    let header = row![
        back,
        Space::new().width(Length::Fill),
        if project.session_count > 0 {
            muted_chip(format!("{}", project.session_count))
        } else {
            Space::new().into()
        },
    ]
    .align_y(Alignment::Center)
    .padding(theme::pad(2.0, 4.0, 6.0, 0.0));

    let repos: Vec<Element<'a, Message>> = project.repos.iter().map(drill_repo).collect();

    let body = column![
        header,
        rule_h(),
        column(repos).spacing(10.0).padding(theme::pad(6.0, 4.0, 10.0, 4.0)),
    ]
    .width(Length::Fill);

    // 不再单独滚动：整体滚动由 AgentsPanel::view 的 scrollable 统一承担
    body.into()
}

fn drill_repo<'a>(repo: &'a RepoNode) -> Element<'a, Message> {
    let lanes: Vec<Element<'a, Message>> = repo.lanes.iter().map(drill_lane).collect();

    column![
        row![
            svg_icon("folder-git", 13.0, theme::TEXT_MUTED),
            text(&repo.label).size(12).font(bold_font()).color(theme::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            muted_chip(format!("{}", repo.session_count)),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center)
        .padding(theme::pad(2.0, 0.0, 2.0, 0.0)),
        column(lanes).spacing(4.0).padding(theme::pad(0.0, 0.0, 0.0, 16.0)),
    ]
    .spacing(4.0)
    .into()
}

fn drill_lane<'a>(lane: &'a LaneGroup) -> Element<'a, Message> {
    let sessions: Vec<Element<'a, Message>> = lane.sessions.iter().map(drill_session).collect();

    column![
        row![
            svg_icon("git-branch", 12.0, theme::TEXT_MUTED),
            text(&lane.label).size(12).color(theme::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            muted_chip(format!("{}", lane.session_count)),
        ]
        .spacing(6.0)
        .align_y(Alignment::Center)
        .padding(theme::pad(2.0, 0.0, 2.0, 0.0)),
        column(sessions)
            .spacing(2.0)
            .padding(theme::pad(0.0, 0.0, 0.0, 16.0)),
    ]
    .spacing(2.0)
    .into()
}

fn drill_session<'a>(s: &'a SessionPreview) -> Element<'a, Message> {
    row![
        color_dot(6.0, theme::TEXT_MUTED),
        svg_icon("message-square", 12.0, theme::TEXT_MUTED),
        text(&s.title).size(12).color(theme::TEXT_PRIMARY),
        Space::new().width(Length::Fill),
        text(&s.last_active).size(10).color(theme::TEXT_MUTED),
    ]
    .spacing(6.0)
    .align_y(Alignment::Center)
    .padding(theme::pad(3.0, 6.0, 3.0, 16.0))
    .into()
}

// ───────────────────────────────────────────────────────────
// 新建弹窗（modal overlay）：对齐 Tauri CreateAgentPopover 视觉
// ───────────────────────────────────────────────────────────

pub fn create_dialog_view<'a>(state: &'a State, kind: CreateDialog) -> Element<'a, Message> {
    let (title, subtitle) = match kind {
        CreateDialog::Agent => ("新建 Agent", "只填昵称即可，ID 自动生成"),
        CreateDialog::Project => ("新建项目", "填写名称后创建项目"),
    };

    let input = text_input("请输入名称…", &state.create_input)
        .width(Length::Fill)
        .on_input(Message::CreateInputChanged)
        .on_submit(Message::ConfirmCreate)
        .style(|_: &iced::Theme, s| {
            let border = match s {
                text_input::Status::Focused { .. } => Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: theme::with_alpha(theme::ACCENT, 0.6),
                },
                _ => Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: theme::SEPARATOR,
                },
            };
            text_input::Style {
                background: Background::Color(theme::BG_MUTED),
                border,
                icon: Color::TRANSPARENT,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT_PRIMARY,
                selection: theme::ACCENT,
            }
        });

    let card_content = column![
        row![
            text(title).size(15).font(bold_font()).color(theme::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            close_x(),
        ]
        .align_y(Alignment::Center),
        text(subtitle).size(11).color(theme::with_alpha(theme::TEXT_MUTED, 0.6)),
        column![
            text("名称").size(12).color(theme::TEXT_PRIMARY),
            input,
        ]
        .spacing(6.0),
        row![
            button(text("取消").size(13).color(theme::TEXT_PRIMARY))
                .style(|_: &iced::Theme, s| {
                    let bg = match s {
                        button::Status::Hovered | button::Status::Pressed => {
                            Some(Background::Color(theme::with_alpha(theme::ACCENT, 0.30)))
                        }
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        text_color: theme::TEXT_MUTED,
                        border: Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: theme::SEPARATOR,
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::CloseCreateDialog),
            button(text("创建").size(13).color(theme::TEXT_ON_ACCENT))
                .style(move |_: &iced::Theme, s| theme::primary_pill(s, theme::ACCENT))
                .on_press(Message::ConfirmCreate),
        ]
        .spacing(8.0)
        .align_y(Alignment::Center),
    ]
    .spacing(14.0);

    // 卡片：用 container 承载（切勿用 button 包裹！否则 text_input / 内部按钮被外层 button 吞掉事件，
    // 导致无法输入、无法点击——这是之前“新建项目卡片”严重 BUG 的根因）。
    // 外层 MouseArea(on_press=Dismiss) 仅用于捕获卡片点击、阻止穿透到背景（Dismiss 为无操作）；
    // 内部 text_input / 取消 / 创建 各自正常工作。
    let card = container(card_content)
        .width(Length::Fixed(360.0))
        .padding(theme::pad(18.0, 18.0, 18.0, 18.0))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::BG_ELEVATED)),
            border: Border {
                radius: theme::CARD_RADIUS.into(),
                width: 1.0,
                color: theme::SEPARATOR,
            },
            shadow: Shadow {
                color: theme::SHADOW_HEAVY,
                offset: Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });
    // 背景层：全屏暗化按钮，点背景即关闭（on_press=CloseCreateDialog）。
    // 关键：卡片（card）是 stack 中与背景【兄弟叠加】的独立元素（后者在上），
    // 因此卡片内的 text_input / 取消 / 创建 按钮均不与背景按钮嵌套，点击互不干扰——
    // 既修复了“button 包裹 text_input 导致无法输入”的旧 BUG，也去掉了包裹整卡
    // 的 MouseArea（避免其对输入框焦点/事件带来任何歧义）。
    // 点卡片空白处会穿透到背景按钮关闭弹窗（通用 modal 习惯）。
    let backdrop = button(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme, _: button::Status| button::Style {
            background: Some(Background::Color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.5,
            })),
            border: Border::default(),
            ..Default::default()
        })
        .on_press(Message::CloseCreateDialog);

    let card_layer = container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    stack![backdrop, card_layer].into()
}

fn close_x<'a>() -> Element<'a, Message> {
    button(svg_icon("x", 13.0, theme::with_alpha(theme::TEXT_MUTED, 0.6)))
        .style(move |_: &iced::Theme, s| {
            let col = match s {
                button::Status::Hovered | button::Status::Pressed => theme::TEXT_PRIMARY,
                _ => theme::with_alpha(theme::TEXT_MUTED, 0.6),
            };
            button::Style {
                background: None,
                text_color: col,
                border: Border::default(),
                ..Default::default()
            }
        })
        .on_press(Message::CloseCreateDialog)
        .into()
}

// ───────────────────────────────────────────────────────────
// 操作按钮（兄弟节点，避免 button 嵌套）
// ───────────────────────────────────────────────────────────

/// Agent 删除按钮：hover 转危险红（对齐 Tauri hover:text-destructive hover:bg-destructive/10）
fn delete_button<'a>(id: String) -> Element<'a, Message> {
    button(svg_icon("trash-2", 14.0, theme::with_alpha(theme::TEXT_MUTED, 0.5)))
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .style(move |_: &iced::Theme, s| {
            let (bg, col) = match s {
                button::Status::Hovered | button::Status::Pressed => (
                    Some(Background::Color(theme::with_alpha(theme::DESTRUCTIVE, 0.12))),
                    theme::DESTRUCTIVE,
                ),
                _ => (None, theme::with_alpha(theme::TEXT_MUTED, 0.5)),
            };
            button::Style {
                background: bg,
                text_color: col,
                border: Border::default(),
                ..Default::default()
            }
        })
        .on_press(Message::DeleteProfile(id))
        .into()
}

/// 项目更多按钮（hover 提亮；对齐 Tauri kebab hover:text-foreground hover:bg-accent/50）
fn kebab_button<'a>(id: String) -> Element<'a, Message> {
    button(svg_icon(
        "ellipsis-vertical",
        15.0,
        theme::with_alpha(theme::TEXT_MUTED, 0.6),
    ))
    .width(Length::Fixed(28.0))
    .height(Length::Fixed(28.0))
    .style(move |_: &iced::Theme, s| {
        let (bg, col) = match s {
            button::Status::Hovered | button::Status::Pressed => (
                Some(Background::Color(theme::with_alpha(theme::ACCENT, 0.30))),
                theme::TEXT_PRIMARY,
            ),
            _ => (None, theme::with_alpha(theme::TEXT_MUTED, 0.6)),
        };
        button::Style {
            background: bg,
            text_color: col,
            border: Border::default(),
            ..Default::default()
        }
    })
    .on_press(Message::DeleteProject(id))
    .into()
}

// ───────────────────────────────────────────────────────────
// 卡片容器样式
// ───────────────────────────────────────────────────────────

/// 通用卡片 chrome：选中 = 10% 淡底 + 45% 描边 + 重投影；未选中 = 浅卡片底 + 30% 描边 + 微投影
/// Agent 与 Project 卡片共用此样式，差异仅由 accent / selected 数据驱动（不重复造轮子）。
fn card_chrome(
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
                    color: theme::with_alpha(accent, 0.45),
                },
                shadow: card_shadow(true),
                ..Default::default()
            }
        } else {
            container::Style {
                background: Some(Background::Color(theme::BG_CARD)),
                border: Border {
                    radius: 10.0.into(),
                    width: 1.0,
                    color: theme::with_alpha(accent, 0.30),
                },
                shadow: card_shadow(false),
                ..Default::default()
            }
        }
    }
}

/// 通用卡片外壳：左侧发光竖条 + 主体(inner) + 整卡点击选中（MouseArea）
/// 尾部操作（删除 / 更多）仅在鼠标悬停【整卡】时出现（hover 切换），平时以等宽占位预留，避免布局抖动。
/// 通过闭包 mk_inner / mk_actions 构建，规避 Element 不可 Clone 的限制；
/// Agent 与 Project 卡片共用此组件——差异完全由传入的数据（accent / 内容 / 操作）决定，不重复造轮子。
fn card_frame<'a>(
    selected: bool,
    accent: Color,
    mk_inner: impl Fn() -> Element<'a, Message> + 'a,
    mk_actions: impl Fn() -> Vec<Element<'a, Message>> + 'a,
    select: Message,
) -> Element<'a, Message> {
    let actions = mk_actions();

    // 构建卡片的闭包（base/top 结构必须完全相同；Element 不可 Clone，故用闭包复用逻辑）
    let mk_card = |current_actions: Vec<Element<'a, Message>>| -> Element<'a, Message> {
        let actions_row: Element<'a, Message> = if current_actions.is_empty() {
            Space::new().into()
        } else {
            row(current_actions).spacing(0.0).align_y(Alignment::Center).into()
        };
        container(
            row![
                accent_bar(selected, accent),
                container(MouseArea::new(mk_inner()).on_press(select.clone()))
                    .padding(theme::pad(0.0, 0.0, 0.0, 7.0))
                    .width(Length::Fill),
                actions_row,
            ]
            .spacing(0.0)
            .align_y(Alignment::Center),
        )
        .padding(theme::pad(0.0, 8.0, 0.0, 0.0))
        .style(card_chrome(selected, accent))
        .into()
    };

    // base：透明占位（同尺寸 28×28 container），保证 base/top 结构一致
    let base_actions: Vec<Element<'a, Message>> = actions
        .iter()
        .map(|_| {
            container(Space::new())
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0))
                .into()
        })
        .collect();

    // hover 实现"鼠标放整卡 → 操作按钮显现"。
    // 根因：iced 0.14 hover 在 base/top 结构/尺寸不一致时会导致卡片在列中塌为 0 而不显示，
    // 因此 base 必须与 top 保持完全相同的子结构（用同尺寸透明 container 占位）。
    hover(mk_card(base_actions), mk_card(actions)).into()
}
