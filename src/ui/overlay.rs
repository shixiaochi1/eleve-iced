// 模态弹窗 —— 对齐 Tauri OverlayView
// 全屏暗化背景（点击关闭）+ 居中卡片：
//   · Settings : panel 模式（左导航 + 右内容，无标题栏）
//   · Theme / About / Model : 带标题栏 + 关闭 X
// 内部点击不关闭（Dismiss 阻止穿透），仅背景 / 关闭按钮 / ESC 关闭。

use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{Element, Background, Length, Border, Alignment, Color};

use crate::ui::{placeholder, Message, Overlay, State, theme};

pub fn view<'a>(state: &'a State, overlay: Overlay) -> Element<'a, Message> {
    let backdrop = button(Space::new().width(Length::Fill).height(Length::Fill))
        .style(|_: &iced::Theme, _s| iced::widget::button::Style {
            background: Some(Background::Color(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.45 })),
            border: Border::default(),
            ..Default::default()
        })
        .on_press(Message::CloseOverlay);

    let card: Element<'a, Message> = match overlay {
        Overlay::Settings => settings_card(state),
        Overlay::Theme => titled_card("主题", placeholder::theme_view(state), 460.0, 420.0),
        Overlay::About => titled_card("关于", placeholder::about_view(state), 460.0, 360.0),
        Overlay::Model => titled_card("选择模型", placeholder::model_view(state), 460.0, 380.0),
    };

    // 居中：用全尺寸容器对齐（非交互，事件穿透到背景）
    let centered = container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    stack![backdrop, centered].into()
}

// ── 带标题栏的卡片 ──
fn titled_card<'a>(title: &'static str, body: Element<'a, Message>, w: f32, h: f32) -> Element<'a, Message> {
    let close_btn = icon_close();

    let header = row![
        text(title).size(16).color(theme::TEXT_PRIMARY).width(Length::Fill),
        close_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(theme::pad(14.0, 16.0, 14.0, 16.0));

    let sep = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::SEPARATOR)),
            ..Default::default()
        });

    let inner = column![
        header,
        sep,
        container(body).width(Length::Fill).height(Length::Fill).padding(16),
    ]
    .spacing(0)
    .height(Length::Fill);

    card_shell(inner.into(), w, h)
}

// ── 设置卡片（panel 模式：无标题栏，左导航+右内容）──
fn settings_card<'a>(state: &'a State) -> Element<'a, Message> {
    let inner = column![
        // 顶部放一个关闭按钮（panel 模式靠背景关闭，这里加 X 便于操作）
        row![
            Space::new().width(Length::Fill).height(Length::Shrink),
            icon_close(),
        ]
        .padding(theme::pad(8.0, 10.0, 0.0, 0.0)),
        placeholder::settings_view(state),
    ]
    .spacing(0)
    .height(Length::Fill);

    card_shell(inner.into(), 880.0, 560.0)
}

// ── 卡片外壳：圆角 + 边框 + 内部点击不关闭 ──
fn card_shell<'a>(inner: Element<'a, Message>, w: f32, h: f32) -> Element<'a, Message> {
    button(inner)
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .padding(0)
        .style(|_: &iced::Theme, _s: iced::widget::button::Status| iced::widget::button::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border {
                radius: theme::CARD_RADIUS.into(),
                width: 1.0,
                color: theme::SEPARATOR,
            },
            ..Default::default()
        })
        // 关键：卡片捕获点击，阻止穿透到背景（Dismiss = 无操作）
        .on_press(Message::Dismiss)
        .into()
}

fn icon_close<'a>() -> Element<'a, Message> {
    button(text("✕").size(14).color(theme::TEXT_MUTED))
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .padding(0)
        .style(|_: &iced::Theme, status| {
            let hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if hovered { Some(Background::Color(theme::BG_HOVER)) } else { None },
                border: Border::default(),
                ..Default::default()
            }
        })
        .on_press(Message::CloseOverlay)
        .into()
}
