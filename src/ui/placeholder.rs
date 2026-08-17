// 占位面板 —— 为尚未深度实现的分区提供「有内容的面板」，区别于纯「待实现」。
// 所有数据均为 mock，不接后端。
//   · section_view : 左面板分区（cron/tools/learning/channels/usage/debug/gateway）
//   · settings_view / theme_view / about_view / model_view : 模态弹窗内容

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Background, Border, Alignment, Color};

use crate::ui::{LeftPanel, Message, State, theme};

// ============================================================
// 左面板分区内容
// ============================================================

pub fn section_view<'a>(state: &'a State, panel: LeftPanel) -> Element<'a, Message> {
    let (title, body) = match panel {
        LeftPanel::Cron => ("定时任务", cron_body()),
        LeftPanel::Tools => ("工具", tools_body()),
        LeftPanel::Learning => ("学习", learn_body()),
        LeftPanel::Channels => ("频道", channels_body()),
        LeftPanel::Usage => ("用量分析", usage_body()),
        LeftPanel::Debug => ("调试", debug_body()),
        LeftPanel::Gateway => ("网关状态", gateway_body(state)),
        _ => ("", column![].into()),
    };

    let header = row![text(title).size(16).color(theme::TEXT_PRIMARY)]
        .padding([14, 18])
        .align_y(Alignment::Center);

    let content = column![header, body].spacing(0);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::card_style())
        .into()
}

fn page_body<'a>(inner: Element<'a, Message>) -> Element<'a, Message> {
    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(theme::pad(0.0, 18.0, 18.0, 18.0))
        .into()
}

// ── 定时任务 ──
fn cron_body<'a>() -> Element<'a, Message> {
    let tasks: Vec<(&str, &str, &str)> = vec![
        ("每日构建", "每天 02:00", "已启用"),
        ("周报生成", "每周一 09:00", "已启用"),
        ("日志清理", "每天 04:30", "已暂停"),
        ("模型健康检查", "每 30 分钟", "已启用"),
    ];
    let rows: Vec<Element<'a, Message>> = tasks
        .iter()
        .map(|(name, time, status)| {
            let on = status == &"已启用";
            row![
                text(*name).size(13).color(theme::TEXT_PRIMARY).width(Length::Fill),
                text(*time).size(12).color(theme::TEXT_MUTED),
                status_pill(*status, on),
            ]
            .spacing(12)
            .padding([10, 12])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    let list = column(rows).spacing(8);
    page_body(scrollable(list).into())
}

// ── 工具 ──
fn tools_body<'a>() -> Element<'a, Message> {
    let tools: Vec<(&str, &str)> = vec![
        ("文件读写", "本地/远程文件读写与编辑"),
        ("代码搜索", "跨项目语义检索"),
        ("终端", "执行 shell 命令"),
        ("网页抓取", "抓取并解析网页内容"),
        ("图像生成", "文生图 / 图生图"),
        ("数据库", "连接并查询数据库"),
        ("翻译", "多语言互译"),
        ("日程", "读写日历与提醒"),
    ];
    let cards: Vec<Element<'a, Message>> = tools
        .chunks(2)
        .map(|chunk| {
            let row_cards: Vec<Element<'a, Message>> = chunk
                .iter()
                .map(|(name, desc)| tool_card(name, desc))
                .collect();
            row(row_cards).spacing(12).into()
        })
        .collect();

    page_body(scrollable(column(cards).spacing(12)).into())
}

fn tool_card<'a>(name: &'a str, desc: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(name).size(13).color(theme::TEXT_PRIMARY),
            text(desc).size(11).color(theme::TEXT_MUTED),
        ]
        .spacing(6)
        .padding(14),
    )
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.11, 0.12, 0.14))),
        border: Border { radius: 10.0.into(), width: 1.0, color: theme::SEPARATOR },
        ..Default::default()
    })
    .into()
}

// ── 学习 ──
fn learn_body<'a>() -> Element<'a, Message> {
    let items: Vec<(&str, &str)> = vec![
        ("Rust 异步编程", "已掌握 80%"),
        ("Tauri 插件开发", "进行中 45%"),
        ("iced 立即模式 GUI", "进行中 30%"),
        ("LLM Agent 编排", "已掌握 60%"),
    ];
    let rows: Vec<Element<'a, Message>> = items
        .iter()
        .map(|(name, prog)| {
            row![
                text(*name).size(13).color(theme::TEXT_PRIMARY).width(Length::Fill),
                progress_bar(prog),
            ]
            .spacing(12)
            .padding([12, 12])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    page_body(scrollable(column(rows).spacing(8)).into())
}

// ── 频道 ──
fn channels_body<'a>() -> Element<'a, Message> {
    let chans: Vec<(&str, &str)> = vec![
        ("# 产品讨论", "128 人在线"),
        ("# 工程群", "64 人在线"),
        ("# 设计评审", "32 人在线"),
        ("# 每日站会", "12 人在线"),
    ];
    let rows: Vec<Element<'a, Message>> = chans
        .iter()
        .map(|(name, count)| {
            row![
                text(*name).size(13).color(theme::TEXT_PRIMARY).width(Length::Fill),
                text(*count).size(11).color(theme::TEXT_MUTED),
            ]
            .spacing(12)
            .padding([12, 12])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    page_body(scrollable(column(rows).spacing(8)).into())
}

// ── 用量分析 ──
fn usage_body<'a>() -> Element<'a, Message> {
    let stats: Vec<(&str, &str)> = vec![
        ("总 Tokens", "1.28M"),
        ("会话数", "342"),
        ("活跃 Agent", "5"),
        ("平均时延", "1.2s"),
    ];
    let cards: Vec<Element<'a, Message>> = stats
        .iter()
        .map(|(label, value)| stat_card(label, value))
        .collect();

    let grid = row(cards).spacing(12).padding(theme::pad(0.0, 0.0, 16.0, 0.0));

    let detail = container(
        column![
            text("Token 消耗趋势（近 7 天）").size(12).color(theme::TEXT_MUTED),
            mini_bars(),
        ]
        .spacing(10),
    )
    .padding(14.0)
    .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.11, 0.12, 0.14))),
        border: Border { radius: 10.0.into(), width: 1.0, color: theme::SEPARATOR },
        ..Default::default()
    });

    page_body(column![grid, detail].into())
}

fn stat_card<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(value).size(20).color(theme::TEXT_PRIMARY),
            text(label).size(11).color(theme::TEXT_MUTED),
        ]
        .spacing(6)
        .padding(14),
    )
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.11, 0.12, 0.14))),
        border: Border { radius: 10.0.into(), width: 1.0, color: theme::SEPARATOR },
        ..Default::default()
    })
    .into()
}

fn mini_bars<'a>() -> Element<'a, Message> {
    let heights = [0.4, 0.6, 0.5, 0.8, 0.7, 0.9, 0.65];
    let bars: Vec<Element<'a, Message>> = heights
        .iter()
        .map(|h| {
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(80.0 * *h))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(Background::Color(theme::ACCENT)),
                    border: Border { radius: 4.0.into(), ..Default::default() },
                    ..Default::default()
                })
                .into()
        })
        .collect();
    row(bars).spacing(8).align_y(Alignment::End).into()
}

// ── 调试 ──
fn debug_body<'a>() -> Element<'a, Message> {
    let logs = [
        "[02:14:03] INFO  bootstrap complete",
        "[02:14:04] DEBUG session manager ready",
        "[02:14:05] INFO  ws server listening on 127.0.0.1:7921",
        "[02:14:07] WARN  provider 'deepseek' latency 320ms",
        "[02:14:09] INFO  tool 'file_read' registered",
        "[02:14:12] ERROR dispatch queue full, retrying",
        "[02:14:15] INFO  recovered, queue drained",
    ];
    let rows: Vec<Element<'a, Message>> = logs
        .iter()
        .map(|line| {
            let color = if line.contains("ERROR") {
                Color::from_rgb(0.85, 0.35, 0.35)
            } else if line.contains("WARN") {
                Color::from_rgb(0.85, 0.62, 0.20)
            } else {
                theme::TEXT_MUTED
            };
            text(*line).size(11).color(color).font(iced::Font::MONOSPACE).into()
        })
        .collect();

    page_body(scrollable(column(rows).spacing(4).padding(12)).into())
}

// ── 网关状态（Logo 按钮）──
fn gateway_body<'a>(_state: &'a State) -> Element<'a, Message> {
    let rows: Vec<Element<'a, Message>> = vec![
        ("网关地址", "127.0.0.1:7921"),
        ("连接状态", "在线"),
        ("协议版本", "v2"),
        ("Ping", "12ms"),
    ]
    .iter()
    .map(|(k, v)| {
        row![
            text(*k).size(13).color(theme::TEXT_MUTED).width(Length::Fill),
            text(*v).size(13).color(theme::TEXT_PRIMARY),
        ]
        .spacing(12)
        .padding([10, 12])
        .align_y(Alignment::Center)
        .into()
    })
    .collect();

    page_body(scrollable(column(rows).spacing(8)).into())
}

// ============================================================
// 通用小部件
// ============================================================

fn status_pill<'a>(label: &'a str, on: bool) -> Element<'a, Message> {
    let color = if on {
        Color::from_rgb(0.30, 0.70, 0.45)
    } else {
        Color::from_rgb(0.55, 0.58, 0.62)
    };
    container(text(label).size(11).color(color))
        .padding([3, 10])
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(Color { r: color.r, g: color.g, b: color.b, a: 0.15 })),
            border: Border { radius: 12.0.into(), width: 0.0, color: Color::TRANSPARENT },
            ..Default::default()
        })
        .into()
}

fn progress_bar<'a>(_label: &'a str) -> Element<'a, Message> {
    container(Space::new().width(Length::Fixed(80.0)).height(Length::Fixed(6.0)))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::SEPARATOR)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

// ============================================================
// 模态弹窗内容：Settings / Theme / About / Model
// ============================================================

/// 设置（panel 模式：左导航 + 右内容贴成一张卡，无标题栏）
pub fn settings_view<'a>(state: &'a State) -> Element<'a, Message> {
    let nav = ["常规", "连接", "模型", "Provider", "记忆", "安全", "语音", "MCP", "工作区", "系统"];
    let nav_col: Vec<Element<'a, Message>> = nav
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let selected = i == 0;
            row![
                text(*g).size(13).color(if selected { theme::ACCENT } else { theme::TEXT_PRIMARY }).width(Length::Fill),
                if selected { text("●").size(10).color(theme::ACCENT) } else { text("").size(10) },
            ]
            .spacing(8)
            .padding([10, 14])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    let toggles = [
        ("自动更新", "auto_update"),
        ("回车发送", "send_on_enter"),
        ("流式输出 tokens", "stream_tokens"),
        ("紧凑面板", "compact_panel"),
        ("声音提醒", "sound_alert"),
    ];
    let toggle_col: Vec<Element<'a, Message>> = toggles
        .iter()
        .map(|(label, key)| setting_row(state, label, key))
        .collect();

    let right = column![
        text("常规").size(15).color(theme::TEXT_PRIMARY),
        Space::new().height(Length::Fixed(8.0)),
        column(toggle_col).spacing(2),
    ]
    .spacing(0)
    .padding(20);

    let left = column(nav_col).spacing(2).padding(theme::pad(12.0, 0.0, 12.0, 0.0));

    row![
        container(left)
            .width(Length::Fixed(180.0))
            .height(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.10, 0.11, 0.13))),
                border: Border { radius: 0.0.into(), width: 0.0, color: Color::TRANSPARENT },
                ..Default::default()
            }),
        container(right).width(Length::Fill).height(Length::Fill),
    ]
    .into()
}

fn setting_row<'a>(state: &'a State, label: &'a str, key: &'a str) -> Element<'a, Message> {
    let on = *state.settings.get(key).unwrap_or(&false);
    let knob = container(Space::new().width(Length::Fixed(14.0)).height(Length::Fixed(14.0)))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(if on { theme::ACCENT } else { theme::SEPARATOR })),
            border: Border { radius: 8.0.into(), ..Default::default() },
            ..Default::default()
        });
    row![
        text(label).size(13).color(theme::TEXT_PRIMARY).width(Length::Fill),
        button(knob)
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(22.0))
            .padding(4)
            .style(move |_: &iced::Theme, _s| iced::widget::button::Style {
                background: Some(Background::Color(if on { theme::ACCENT } else { Color::from_rgb(0.25, 0.26, 0.28) })),
                border: Border { radius: 11.0.into(), ..Default::default() },
                ..Default::default()
            })
            .on_press(Message::ToggleSetting(key.to_string())),
    ]
    .spacing(12)
    .padding([10, 4])
    .align_y(Alignment::Center)
    .into()
}

pub fn theme_view<'a>(_state: &'a State) -> Element<'a, Message> {
    let themes = ["暗色（默认）", "暗色蓝", "午夜", "石墨"];
    let rows: Vec<Element<'a, Message>> = themes
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = i == 0;
            row![
                text(*t).size(13).color(if selected { theme::ACCENT } else { theme::TEXT_PRIMARY }).width(Length::Fill),
                if selected { text("●").size(12).color(theme::ACCENT) } else { text("").size(12) },
            ]
            .spacing(8)
            .padding([12, 16])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    container(column(rows).spacing(2))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn about_view<'a>(_state: &'a State) -> Element<'a, Message> {
    let content = column![
        text("ELEVE Agent").size(18).color(theme::TEXT_PRIMARY),
        text("原生桌面端 · iced 复刻版").size(12).color(theme::TEXT_MUTED),
        Space::new().height(Length::Fixed(12.0)),
        text("版本 0.1.0 (iced)").size(12).color(theme::TEXT_MUTED),
        text("基于 Rust + iced 0.14 构建").size(12).color(theme::TEXT_MUTED),
        text("单进程 · 低内存 · 原生体验").size(12).color(theme::TEXT_MUTED),
    ]
    .spacing(6)
    .padding([16, 16])
    .align_x(Alignment::Start);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn model_view<'a>(_state: &'a State) -> Element<'a, Message> {
    let models = ["qwen3.7-plus", "deepseek-v3", "claude-sonnet", "gpt-4o", "glm-4.5"];
    let rows: Vec<Element<'a, Message>> = models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let selected = i == 0;
            row![
                text(*m).size(13).color(if selected { theme::ACCENT } else { theme::TEXT_PRIMARY }).width(Length::Fill),
                if selected { text("●").size(12).color(theme::ACCENT) } else { text("").size(12) },
            ]
            .spacing(8)
            .padding([12, 16])
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    container(column(rows).spacing(2))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
