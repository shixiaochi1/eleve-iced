// 右侧抽屉 —— 对齐 Tauri RightSidebarTabs + Pane(right)
// 4 个功能界面：文件 / 终端 / 预览 / 产物（Files / Terminal / Preview / Artifacts）
// 顶部 tab 栏（激活态强调色 + 底边框），右侧关闭 X；下方为对应内容。

use iced::widget::{button, column, container, row, scrollable, text, Svg, Space, rule};
use iced::{Element, Background, Length, Border, Alignment, Color, Padding};

use crate::ui::{file_browser, Message, RightTab, State, theme};

const ASSET_BASE: &str = env!("CARGO_MANIFEST_DIR");
fn asset_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(ASSET_BASE).join("assets/icons").join(format!("{}.svg", name))
}

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    let tabs = tab_bar(state);

    let content: Element<'a, Message> = match state.right_tab {
        RightTab::Files => file_browser::view(state),
        RightTab::Terminal => terminal_view(),
        RightTab::Preview => preview_view(),
        RightTab::Artifacts => artifacts_view(),
    };

    // 单列：顶部 tab 条带（flush） + 正文。整张卡片只在外层。
    let col = column![
        tabs,
        container(content)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(0)
    .height(Length::Fill);

    // ── 单一卡片外壳（与 chat_area 同约定：BG_CARD + 圆角 + 无边框）──
    container(col)
        .width(Length::Fixed(theme::RIGHT_DRAWER_WIDTH))
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

// ── Tab 栏（顶部条带：透明背景 + 底边线，对齐 chat_area 的 ToolStatusBar）──
fn tab_bar<'a>(state: &'a State) -> Element<'a, Message> {
    let tabs: Vec<(&str, &str, RightTab)> = vec![
        ("file", "文件", RightTab::Files),
        ("terminal", "终端", RightTab::Terminal),
        ("globe", "预览", RightTab::Preview),
        ("box", "产物", RightTab::Artifacts),
    ];

    let mut items: Vec<Element<'a, Message>> = Vec::new();
    for (icon, label, tab) in tabs {
        let is_active = state.right_tab == tab;
        items.push(tab_button(icon, label, is_active, tab));
    }

    // 关闭按钮（✕，右侧）
    let close_btn = button(text("✕").size(13).color(theme::text_muted()))
        .width(Length::Fixed(26.0))
        .height(Length::Fixed(26.0))
        .padding(0)
        .style(|_: &iced::Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered { Some(Background::Color(theme::bg_hover())) } else { None },
                border: Border::default(),
                ..Default::default()
            }
        })
        .on_press(Message::CloseRight);

    let bar = row![
        row(items).spacing(0).align_y(Alignment::Center),
        Space::new().width(Length::Fill).height(Length::Shrink),
        close_btn,
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .padding(Padding::new(0.0).right(8.0).left(8.0));

    let separator = rule::horizontal(1.0)
        .style(|_: &iced::Theme| rule::Style {
            color: theme::separator(),
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        });

    // 顶部条带：内容(fill) + 底边线；整体 40px，无独立背景（卡片提供背景）
    column![
        container(bar).width(Length::Fill).height(Length::Fill),
        separator,
    ]
    .height(Length::Fixed(40.0))
    .spacing(0)
    .into()
}

fn tab_button<'a>(icon: &'static str, label: &'static str, is_active: bool, tab: RightTab) -> Element<'a, Message> {
    let color = if is_active { theme::accent() } else { theme::text_muted() };
    let btn = button(
        row![
            Svg::from_path(asset_path(icon))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(move |_: &iced::Theme, _| iced::widget::svg::Style { color: Some(color) }),
            text(label).size(12).color(color),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .style(move |_: &iced::Theme, _status| iced::widget::button::Style {
        background: None,
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    })
    .on_press(Message::OpenRightTab(tab));

    // 激活态底边框
    let col = if is_active {
        column![
            btn,
            container(Space::new().width(Length::Fill).height(Length::Fixed(2.0)))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(Background::Color(theme::accent())),
                    ..Default::default()
                }),
        ]
    } else {
        column![btn]
    };
    col.into()
}

// ── 终端（mock）──
fn terminal_view<'a>() -> Element<'a, Message> {
    let lines = [
        "$ eleve --status",
        "● gateway online  (127.0.0.1:7921)",
        "$ cargo build --release",
        "   Compiling eleve-iced v0.1.0",
        "    Finished in 12.3s",
        "$ _",
    ];
    let rows: Vec<Element<'a, Message>> = lines
        .iter()
        .map(|l| text(*l).size(12).color(theme::text_primary()).font(iced::Font::MONOSPACE).into())
        .collect();

    container(scrollable(column(rows).spacing(4).padding(12)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.06, 0.07, 0.08))),
            // 只圆底部，贴合外层卡片的圆角
            border: Border {
                radius: iced::border::Radius::default().bottom(theme::CARD_RADIUS),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

// ── 预览（mock）──
fn preview_view<'a>() -> Element<'a, Message> {
    let inner = container(
        column![
            Svg::from_path(asset_path("globe"))
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(40.0))
                .style(|_: &iced::Theme, _| iced::widget::svg::Style { color: Some(theme::text_muted()) }),
            text("预览中心").size(14).color(theme::text_primary()),
            text("选择一个产物或文件以在此预览").size(12).color(theme::text_muted()),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center);

    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ── 产物（mock）──
fn artifacts_view<'a>() -> Element<'a, Message> {
    let arts: Vec<(&str, &str)> = vec![
        ("report.md", "Markdown · 12 KB"),
        ("chart.png", "图片 · 84 KB"),
        ("summary.json", "JSON · 3 KB"),
        ("build.log", "日志 · 21 KB"),
    ];
    let rows: Vec<Element<'a, Message>> = arts
        .iter()
        .map(|(name, meta)| {
            row![
                Svg::from_path(asset_path("box"))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(|_: &iced::Theme, _| iced::widget::svg::Style { color: Some(theme::text_muted()) }),
                text(*name).size(13).color(theme::text_primary()).width(Length::Fill),
                text(*meta).size(11).color(theme::text_muted()),
            ]
            .spacing(10)
            .padding([10, 12])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    container(scrollable(column(rows).spacing(6).padding(8)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
