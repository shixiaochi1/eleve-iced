// ELEVE Iced - Native GUI with Iced 0.14
// 1+3布局：1主窗体(无边框+自绘标题栏) + 3卡片

mod ui;

use iced::Element;
use iced::Task;
use iced::Color;
use std::borrow::Cow;
use iced_window_chrome::{ChromeSettings, Event, WindowsChromeSettings, WindowCornerPreference};
use tracing_subscriber;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    iced::application(boot, update, view)
        .theme(iced::Theme::Dark)
        .window(window_settings())
        .subscription(subscription)
        .run()
}

fn boot() -> (State, Task<Message>) {
    let (ui_state, ui_task) = ui::new();
    let chrome = make_chrome_settings();

    (
        State { ui: Some(ui_state), chrome: chrome.clone() },
        Task::batch([
            iced_window_chrome::apply_to_latest::<Message>(chrome),
            ui_task.map(Message::Ui),
            load_chinese_fonts(),
        ]),
    )
}

/// 加载系统中文字体，使 CJK 字形可被渲染。
///
/// iced 默认只内置 Iced-Icons / FiraSans（无中文字形），
/// 因此中文会变成方块。这里把 Windows 自带的微软雅黑（常规 + 粗体，
/// 同文件内含 "Microsoft YaHei" 与 "Microsoft YaHei UI" 两个字面）
/// 注册进 cosmic-text 字体库；对 Han 脚本的回退目标正是
/// "Microsoft YaHei UI"，从而中文自动走雅黑、拉丁文仍走默认字体。
fn load_chinese_fonts() -> Task<Message> {
    // 非 Windows 或字体缺失时静默跳过，不影响其它功能。
    const CANDIDATES: &[&str] = &[
        "C:\\Windows\\Fonts\\msyh.ttc",   // 常规 + UI 常规
        "C:\\Windows\\Fonts\\msyhbd.ttc", // 粗体 + UI 粗体
    ];

    let mut loads: Vec<Task<Message>> = Vec::new();
    for path in CANDIDATES {
        if let Ok(bytes) = std::fs::read(path) {
            // iced::font::load 需要 'static 生命周期的字节，故泄漏所有权。
            let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            loads.push(iced::font::load(Cow::Borrowed(leaked)).map(|_| Message::FontLoaded));
        }
    }

    if loads.is_empty() {
        Task::none()
    } else {
        Task::batch(loads)
    }
}

/// 配置系统原生窗体的标题栏，使其与背板颜色一致，视觉上融为一体
fn make_chrome_settings() -> ChromeSettings {
    // 背板颜色
    let backboard = Color::from_rgb(0.08, 0.09, 0.10);

    ChromeSettings {
        windows: WindowsChromeSettings {
            // 保留标题栏（caption区域），否则关闭/最小化/最大化按钮也会消失
            caption: true,
            // 保留边框（用于拖拽缩放）
            border: true,
            // 保留关闭/最小化/最大化按钮
            buttons: iced_window_chrome::CaptionButtons::default(),
            // Win11 圆角
            corner_preference: Some(WindowCornerPreference::Round),
            // 边框颜色 = 背板色（消除分界线）
            border_color: Some(backboard),
            // 标题栏背景色 = 背板色（融为一体）
            title_background_color: Some(backboard),
            // 标题文字颜色 = 背板色（不可见）
            title_text_color: Some(backboard),
            // 不使用系统 backdrop
            backdrop: None,
        },
        macos: Default::default(),
        linux: Default::default(),
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Ui(msg) => ui::update(state.ui.as_mut().unwrap(), msg).map(Message::Ui),
        Message::Chrome(event) => iced_window_chrome::handle(event),
        Message::FontLoaded => Task::none(),
    }
}

fn view(state: &State) -> Element<'_, Message> {
    ui::view(state.ui.as_ref().unwrap()).map(Message::Ui)
}

fn subscription(state: &State) -> iced::Subscription<Message> {
    iced_window_chrome::subscription(state.chrome.clone()).map(Message::Chrome)
}

fn window_settings() -> iced::window::Settings {
    let mut settings = iced::window::Settings::default();
    settings.size = iced::Size::new(1280.0, 800.0);
    settings
}

#[derive(Debug, Clone)]
pub enum Message {
    Ui(ui::Message),
    Chrome(Event),
    FontLoaded,
}

pub struct State {
    ui: Option<ui::State>,
    chrome: ChromeSettings,
}
