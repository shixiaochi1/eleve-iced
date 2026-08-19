// ============================================================
// Theme — macOS 风格配色系统（对齐 Tauri derive.ts）
// 仅 2 个用户参数：accent(主题色) + appearance(外观)
// 所有中性色从 accent 色相派生，换色整体系色联动；
// 灰度(Graphite)主题走 macOS 语义（主按钮实底加重）。
// 通过全局 CURRENT 调色板 + 访问函数暴露，供各组件动态取色。
// ============================================================

use iced::{Border, Background, Color, Padding, Shadow, Vector, widget::container::Style, widget::button::Status};
use std::sync::{LazyLock, Mutex};

// ─── 用户可控参数 ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
    Auto,  // 暂按深色近似
    Glass, // 暂按深色近似（玻璃透明度后续接入窗口合成）
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontScale {
    Small,
    Medium,
    Large,
}

/// 8 个 macOS 标准预设主题色（对齐 Tauri ACCENT_COLORS）
pub const ACCENT_PRESETS: &[(&str, &str)] = &[
    ("蓝色", "#007AFF"),
    ("紫色", "#AF52DE"),
    ("粉色", "#FF2D55"),
    ("红色", "#FF3B30"),
    ("橙色", "#FF9500"),
    ("黄色", "#FFCC00"),
    ("绿色", "#34C759"),
    ("石墨色", "#8E8E93"),
];

/// 🔴 默认主题 = 灰色（Graphite 风格）：安装首次打开默认灰，不抢眼
pub const DEFAULT_ACCENT: &str = "#8E8E93";
pub const DEFAULT_APPEARANCE: Appearance = Appearance::Dark;
pub const DEFAULT_FONT_SCALE: FontScale = FontScale::Medium;

// ─── 调色板 ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Palette {
    pub bg_backboard: Color,
    pub bg_card: Color,
    pub bg_hover: Color,
    pub bg_muted: Color,
    pub bg_elevated: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub text_on_accent: Color,
    pub accent: Color, // = Tauri primary（主按钮实底）
    pub accent_hover: Color,
    pub separator: Color, // 边框
    pub input: Color,
    pub ring: Color,
    pub destructive: Color,
    pub destructive_fg: Color,
    pub sidebar_bg: Color,
    pub sidebar_border: Color,
    pub user_bubble: Color,
    pub user_bubble_border: Color,
    pub selection: Color,
    pub warm_accent: Color,
    pub shadow: Color,
    pub shadow_heavy: Color,
    pub inline_code_bg: Color,
    pub inline_code_border: Color,
    pub inline_code_fg: Color,
    pub semantic_red: Color,
    pub semantic_orange: Color,
    pub semantic_yellow: Color,
    pub semantic_green: Color,
    pub semantic_cyan: Color,
    pub semantic_blue: Color,
    pub semantic_purple: Color,
    pub semantic_pink: Color,
    pub is_dark: bool,
}

// ─── 颜色工具 ─────────────────────────────────────────────────

pub fn col(hex: &str) -> Color {
    let (r, g, b) = hex_rgb(hex);
    Color { r, g, b, a: 1.0 }
}
pub fn col3(rgb: (f32, f32, f32)) -> Color {
    Color { r: rgb.0, g: rgb.1, b: rgb.2, a: 1.0 }
}
pub fn rgba(rgb: (f32, f32, f32), a: f32) -> Color {
    Color { r: rgb.0, g: rgb.1, b: rgb.2, a }
}
pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

fn hex_rgb(hex: &str) -> (f32, f32, f32) {
    let c = hex.trim_start_matches('#');
    let r = i64::from_str_radix(&c[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = i64::from_str_radix(&c[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = i64::from_str_radix(&c[4..6], 16).unwrap_or(0) as f32 / 255.0;
    (r, g, b)
}

/// HSL → RGB（h 0..360，s,l 0..100），返回 0..1 三元组
fn hsl(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let s = s / 100.0;
    let l = l / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (r + m, g + m, b + m)
}

fn hue_of((r, g, b): (f32, f32, f32)) -> f32 {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max == min {
        return 220.0; // 无饱和 → 默认冷蓝灰
    }
    let d = max - min;
    if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
    } else if max == g {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    }
}

/// 灰度判定：RGB 通道最大差 < 12（0..255）视为无色相灰
fn is_gray((r, g, b): (f32, f32, f32)) -> bool {
    let r = (r * 255.0) as i32;
    let g = (g * 255.0) as i32;
    let b = (b * 255.0) as i32;
    r.max(g).max(b) - r.min(g).min(b) < 12
}

/// 降低饱和度（factor 0–1；0.85 = 饱和度 ×0.85）
fn desaturate((r, g, b): (f32, f32, f32), factor: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max == min {
        return (r, g, b);
    }
    let d = max - min;
    let l = (max + min) / 2.0;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let hh = hue_of((r, g, b));
    hsl(hh, (s * factor).max(0.0) * 100.0, l * 100.0)
}

/// WCAG 相对亮度（0–1）
fn rel_lum((r, g, b): (f32, f32, f32)) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// 深色模式下若主题色过暗（亮度 < 0.179）则调亮 1.4×
fn brighten_dark((r, g, b): (f32, f32, f32)) -> (f32, f32, f32) {
    if rel_lum((r, g, b)) < 0.179 {
        ((r * 1.4).min(1.0), (g * 1.4).min(1.0), (b * 1.4).min(1.0))
    } else {
        (r, g, b)
    }
}

/// 色相偏移（用于 warmAccent）
fn shift_hue((r, g, b): (f32, f32, f32), deg: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let s = if max == min {
        0.0
    } else if l > 0.5 {
        (max - min) / (2.0 - max - min)
    } else {
        (max - min) / (max + min)
    };
    let mut hh = if max == min {
        0.0
    } else if max == r {
        ((g - b) / (max - min) + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    hh = (hh + deg / 360.0 + 1.0) % 1.0;
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hr = |p: f32, q: f32, t: f32| -> f32 {
        let mut t = t;
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * ((2.0 / 3.0) - t) * 6.0
        } else {
            p
        }
    };
    (hr(p, q, hh + 1.0 / 3.0), hr(p, q, hh), hr(p, q, hh - 1.0 / 3.0))
}

/// 两色线性混合（t=0→a，t=1→b）
fn mix(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// 8 语义色（macOS 标准，Light/Dark 独立）；顺序 red/orange/yellow/green/cyan/blue/purple/pink
fn semantic(is_dark: bool) -> [Color; 8] {
    if is_dark {
        [
            col("#FF453A"),
            col("#FF9F0A"),
            col("#FFD60A"),
            col("#30D158"),
            col("#64D2FF"),
            col("#0A84FF"),
            col("#BF5AF2"),
            col("#FF375F"),
        ]
    } else {
        [
            col("#FF3B30"),
            col("#FF9500"),
            col("#FFCC00"),
            col("#34C759"),
            col("#5AC8FA"),
            col("#007AFF"),
            col("#AF52DE"),
            col("#FF2D55"),
        ]
    }
}

// ─── 核心派生 ─────────────────────────────────────────────────

fn derive(is_dark: bool, accent_hex: &str) -> Palette {
    let ar = hex_rgb(accent_hex);
    let (adj, h, gray) = if is_dark {
        let adj = brighten_dark(ar);
        (adj, hue_of(adj), is_gray(adj))
    } else {
        (ar, hue_of(ar), is_gray(ar))
    };
    // 灰度主题：主按钮实底走 Graphite 加重（浅=深灰/黑，深=浅灰）
    let base = if gray {
        if is_dark {
            hex_rgb("#E5E5E7")
        } else {
            hex_rgb("#2C2C2E")
        }
    } else {
        adj
    };
    let sem = semantic(is_dark);

    // (背板, 侧栏, 卡片, 弹层, 次文字, 主文字, secondaryα, accentα, borderα, inputα,
    //  userBubbleα, userBubbleBorderα, inlineCodeBgα, inlineCodeBorderα, selectionα,
    //  shadowα, shadowHeavyα, bgHoverMix, bgMutedL)
    let (backboard, sidebar, card, popover, muted, fg, _sec_a, _acc_a, border_a, input_a, ub_a, ubb_a, icbg_a, icb_a, sel_a, sh_a, shh_a, hov_m, mut_l) =
        if is_dark {
            (
                hsl(h, 14.0, 11.0),
                hsl(h, 7.0, 15.0),
                hsl(h, 10.0, 18.0),
                hsl(h, 12.0, 23.0),
                hsl(h, 5.0, 60.0),
                hsl(h, 4.0, 96.0),
                0.16, 0.10, 0.10, 0.08, 0.12, 0.22, 0.06, 0.10, 0.30, 0.30, 0.45, 0.10, 21.0,
            )
        } else {
            (
                hsl(h, 10.0, 92.0),
                hsl(h, 5.0, 94.0),
                hsl(h, 8.0, 100.0),
                hsl(h, 8.0, 100.0),
                hsl(h, 4.0, 55.0),
                hsl(h, 8.0, 12.0),
                0.10, 0.07, 0.08, 0.06, 0.08, 0.18, 0.05, 0.08, 0.25, 0.10, 0.18, 0.04, 97.0,
            )
        };

    let primary = if gray { base } else { desaturate(adj, 0.85) };
    let primary_fg = if is_dark { backboard } else { (1.0, 1.0, 1.0) };
    let bg_hover = mix(card, fg, hov_m);
    let bg_muted = if is_dark {
        hsl(h, 10.0, mut_l)
    } else {
        hsl(h, 4.0, mut_l)
    };
    let shadow_base = if is_dark {
        hsl(h, 10.0, 3.0)
    } else {
        hsl(h, 10.0, 15.0)
    };
    let sidebar_border_a = if is_dark { 0.08 } else { 0.06 };

    Palette {
        bg_backboard: col3(backboard),
        bg_card: col3(card),
        bg_hover: col3(bg_hover),
        bg_muted: col3(bg_muted),
        bg_elevated: col3(popover),
        text_primary: col3(fg),
        text_muted: col3(muted),
        text_on_accent: col3(primary_fg),
        accent: col3(primary),
        accent_hover: col3(mix(primary, (1.0, 1.0, 1.0), 0.10)),
        separator: rgba(fg, border_a),
        input: rgba(fg, input_a),
        ring: col3(base),
        destructive: sem[0],
        destructive_fg: Color::WHITE,
        sidebar_bg: col3(sidebar),
        sidebar_border: rgba(fg, sidebar_border_a),
        user_bubble: rgba(base, ub_a),
        user_bubble_border: rgba(base, ubb_a),
        selection: rgba(base, sel_a),
        warm_accent: col3(shift_hue(adj, -15.0)),
        shadow: rgba(shadow_base, sh_a),
        shadow_heavy: rgba(shadow_base, shh_a),
        inline_code_bg: rgba(fg, icbg_a),
        inline_code_border: rgba(fg, icb_a),
        inline_code_fg: rgba(fg, 0.88),
        semantic_red: sem[0],
        semantic_orange: sem[1],
        semantic_yellow: sem[2],
        semantic_green: sem[3],
        semantic_cyan: sem[4],
        semantic_blue: sem[5],
        semantic_purple: sem[6],
        semantic_pink: sem[7],
        is_dark,
    }
}

// ─── 全局当前调色板 ───────────────────────────────────────────

static CURRENT: LazyLock<Mutex<Palette>> = LazyLock::new(|| Mutex::new(derive(true, DEFAULT_ACCENT)));
static FONT_SCALE_F: Mutex<f32> = Mutex::new(1.0);

/// 应用主题：重算并写入全局调色板，同时记录字号倍率
pub fn apply(accent: &str, appearance: Appearance, fs: FontScale) {
    let is_dark = !matches!(appearance, Appearance::Light);
    let p = derive(is_dark, accent);
    *CURRENT.lock().unwrap() = p;
    *FONT_SCALE_F.lock().unwrap() = match fs {
        FontScale::Small => 0.875,
        FontScale::Medium => 1.0,
        FontScale::Large => 1.125,
    };
}

pub fn palette() -> Palette {
    CURRENT.lock().unwrap().clone()
}
pub fn is_dark() -> bool {
    CURRENT.lock().unwrap().is_dark
}
/// 字号缩放：base × 当前倍率（对齐 Tauri --dt-base-size）
pub fn ts(base: f32) -> f32 {
    base * *FONT_SCALE_F.lock().unwrap()
}

// ─── 调色板访问函数（组件统一从此取色，实现动态主题）───────────

pub fn bg_backboard() -> Color { CURRENT.lock().unwrap().bg_backboard }
pub fn bg_card() -> Color { CURRENT.lock().unwrap().bg_card }
pub fn bg_hover() -> Color { CURRENT.lock().unwrap().bg_hover }
pub fn bg_muted() -> Color { CURRENT.lock().unwrap().bg_muted }
pub fn bg_elevated() -> Color { CURRENT.lock().unwrap().bg_elevated }
pub fn text_primary() -> Color { CURRENT.lock().unwrap().text_primary }
pub fn text_muted() -> Color { CURRENT.lock().unwrap().text_muted }
pub fn text_on_accent() -> Color { CURRENT.lock().unwrap().text_on_accent }
pub fn accent() -> Color { CURRENT.lock().unwrap().accent }
pub fn accent_hover() -> Color { CURRENT.lock().unwrap().accent_hover }
pub fn separator() -> Color { CURRENT.lock().unwrap().separator }
pub fn input() -> Color { CURRENT.lock().unwrap().input }
pub fn ring() -> Color { CURRENT.lock().unwrap().ring }
pub fn destructive() -> Color { CURRENT.lock().unwrap().destructive }
pub fn destructive_fg() -> Color { CURRENT.lock().unwrap().destructive_fg }
pub fn sidebar_bg() -> Color { CURRENT.lock().unwrap().sidebar_bg }
pub fn sidebar_border() -> Color { CURRENT.lock().unwrap().sidebar_border }
pub fn user_bubble() -> Color { CURRENT.lock().unwrap().user_bubble }
pub fn user_bubble_border() -> Color { CURRENT.lock().unwrap().user_bubble_border }
pub fn selection() -> Color { CURRENT.lock().unwrap().selection }
pub fn warm_accent() -> Color { CURRENT.lock().unwrap().warm_accent }
pub fn shadow() -> Color { CURRENT.lock().unwrap().shadow }
pub fn shadow_heavy() -> Color { CURRENT.lock().unwrap().shadow_heavy }
pub fn inline_code_bg() -> Color { CURRENT.lock().unwrap().inline_code_bg }
pub fn inline_code_border() -> Color { CURRENT.lock().unwrap().inline_code_border }
pub fn inline_code_fg() -> Color { CURRENT.lock().unwrap().inline_code_fg }
pub fn semantic_red() -> Color { CURRENT.lock().unwrap().semantic_red }
pub fn semantic_orange() -> Color { CURRENT.lock().unwrap().semantic_orange }
pub fn semantic_yellow() -> Color { CURRENT.lock().unwrap().semantic_yellow }
pub fn semantic_green() -> Color { CURRENT.lock().unwrap().semantic_green }
pub fn semantic_cyan() -> Color { CURRENT.lock().unwrap().semantic_cyan }
pub fn semantic_blue() -> Color { CURRENT.lock().unwrap().semantic_blue }
pub fn semantic_purple() -> Color { CURRENT.lock().unwrap().semantic_purple }
pub fn semantic_pink() -> Color { CURRENT.lock().unwrap().semantic_pink }

// ─── 解析 / 工具（保留）──────────────────────────────────────

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

/// Agent 强调色：有主题色用主题色，否则回退全局 accent
pub fn accent_of(color: &Option<String>) -> Color {
    color.as_deref().and_then(parse_hex).unwrap_or_else(accent)
}

// ─── 尺寸常量（与主题无关，保留）──────────────────────────────

pub const CARD_RADIUS: f32 = 12.0;
pub const CARD_GAP: f32 = 8.0;
pub const LEFT_PANEL_WIDTH: f32 = 280.0;
pub const RIGHT_DRAWER_WIDTH: f32 = 380.0;
pub const ICON_BAR_WIDTH: f32 = 60.0;
pub const ICON_BTN_SIZE: f32 = 40.0;
pub const ICON_SIZE: f32 = 20.0;
pub const NAV_ICON_PADDING: f32 = 10.0; // (ICON_BTN_SIZE - ICON_SIZE) / 2
pub const ICON_BAR_PADDING: f32 = 8.0;  // top/bottom of icon bar content

/// 四方向 padding 构造（iced 0.14 的 Padding 不支持 [t,r,b,l] 四元素数组字面量）
pub fn pad(t: f32, r: f32, b: f32, l: f32) -> Padding {
    Padding { top: t, right: r, bottom: b, left: l }
}

// ─── 容器 / 按钮样式（读全局调色板）────────────────────────────

// Container style shared by all cards
pub fn card_style() -> impl Fn(&iced::Theme) -> Style {
    move |_: &iced::Theme| Style {
        background: Some(Background::Color(bg_card())),
        border: Border {
            radius: CARD_RADIUS.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

// 实心渐变药丸按钮样式（近似 Tauri 的 from-primary to-primary/90 渐变 + 发光阴影）
pub fn primary_pill(status: Status) -> iced::widget::button::Style {
    let a = accent();
    let (bg, shadow) = match status {
        Status::Pressed => (
            with_alpha(a, 0.85),
            Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
        ),
        Status::Hovered => (
            a,
            Shadow {
                color: with_alpha(a, 0.35),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        ),
        _ => (
            with_alpha(a, 0.95),
            Shadow {
                color: with_alpha(a, 0.25),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
        ),
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: text_on_accent(),
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
