// ============================================================
// Theme — 全局颜色、尺寸、样式常量（单一真相源）
// ============================================================

use iced::{Color, Border, Background};

// Colors
pub const BG_BACKBOARD: Color = Color::from_rgb(0.08, 0.09, 0.10);
pub const BG_CARD: Color = Color::from_rgb(0.14, 0.15, 0.17);
pub const BG_HOVER: Color = Color::from_rgb(0.18, 0.19, 0.21);
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.93, 0.95);
pub const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.58);
pub const TEXT_ON_ACCENT: Color = Color::WHITE;
pub const ACCENT: Color = Color::from_rgb(0.38, 0.53, 0.89);
pub const ACCENT_HOVER: Color = Color { r: 0.38, g: 0.53, b: 0.89, a: 0.7 };
pub const SEPARATOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);

// Dimensions
pub const CARD_RADIUS: f32 = 12.0;
pub const CARD_GAP: f32 = 8.0;
pub const LEFT_PANEL_WIDTH: f32 = 260.0;
pub const DRAWER_WIDTH: f32 = 280.0;
pub const ICON_BAR_WIDTH: f32 = 60.0;
pub const ICON_BTN_SIZE: f32 = 40.0;
pub const ICON_SIZE: f32 = 20.0;
pub const LOGO_ICON_SIZE: f32 = 24.0;
pub const NAV_ICON_PADDING: f32 = 10.0; // (ICON_BTN_SIZE - ICON_SIZE) / 2
pub const ICON_BAR_PADDING: f32 = 8.0;  // top/bottom of icon bar content

// Container style shared by all cards
pub fn card_style() -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    |_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_CARD)),
        border: Border {
            radius: CARD_RADIUS.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}
