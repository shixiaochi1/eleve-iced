// ============================================================
// Theme — 全局颜色、尺寸、样式常量（单一真相源）
// ============================================================

use iced::{Color, Border, Background, Padding};

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
pub const BG_MUTED: Color = Color::from_rgb(0.20, 0.21, 0.23); // 图标芯片 / 次级底色
pub const BG_ELEVATED: Color = Color::from_rgb(0.17, 0.18, 0.20); // 弹窗/浮层底色（高于卡片）
pub const DESTRUCTIVE: Color = Color::from_rgb(0.86, 0.27, 0.27); // 删除/危险操作（对齐 Tauri --theme-destructive）
pub const SHADOW_HEAVY: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.45); // 选中卡片投影（对齐 Tauri --theme-shadow-color-heavy）
#[allow(dead_code)] // 激活会话标记（对齐 Tauri accent-orange），聊天/会话高亮将复用
pub const ACCENT_ORANGE: Color = Color::from_rgb(0.95, 0.62, 0.20); // 激活会话标记（对齐 Tauri accent-orange）

// ── 颜色工具 ──

/// 解析 `#RRGGBB` → iced Color；非法返回 None
pub fn parse_hex(hex: &str) -> Option<Color> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).ok()? as f32 / 255.0;
    Some(Color::from_rgb(r, g, b))
}

/// 保留 rgb，仅覆盖 alpha
pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

/// Agent 强调色：有主题色用主题色，否则回退全局 ACCENT
pub fn accent_of(color: &Option<String>) -> Color {
    color.as_deref().and_then(parse_hex).unwrap_or(ACCENT)
}

// Dimensions
pub const CARD_RADIUS: f32 = 12.0;
pub const CARD_GAP: f32 = 8.0;
pub const LEFT_PANEL_WIDTH: f32 = 280.0;
pub const RIGHT_DRAWER_WIDTH: f32 = 380.0;
pub const ICON_BAR_WIDTH: f32 = 60.0;
pub const ICON_BTN_SIZE: f32 = 40.0;
pub const ICON_SIZE: f32 = 20.0;
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

// 四方向 padding 构造（iced 0.14 的 Padding 不支持 [t,r,b,l] 四元素数组字面量）
pub fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
    Padding { top: t, right: r, bottom: b, left: l }
}

// 实心渐变药丸按钮样式（近似 Tauri 的 from-primary to-primary/90 渐变 + 发光阴影）
pub fn primary_pill(
    status: iced::widget::button::Status,
    accent: Color,
) -> iced::widget::button::Style {
    let (bg, shadow) = match status {
        iced::widget::button::Status::Pressed => (
            with_alpha(accent, 0.85),
            iced::Shadow {
                color: Color::TRANSPARENT,
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
        ),
        iced::widget::button::Status::Hovered => (
            accent,
            iced::Shadow {
                color: with_alpha(accent, 0.35),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        ),
        _ => (
            with_alpha(accent, 0.95),
            iced::Shadow {
                color: with_alpha(accent, 0.25),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
        ),
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT_ON_ACCENT,
        border: Border {
            // 顶部高光描边（近似 Tauri 的 inset 0 1px 0 rgba(255,255,255,0.25) 玻璃质感）
            radius: 999.0.into(),
            width: 1.0,
            color: with_alpha(Color::WHITE, 0.18),
        },
        shadow,
        ..Default::default()
    }
}
