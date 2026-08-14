mod chat_area;
mod drawer;
mod icon_bar;
mod left_panel;
mod theme;

use iced::widget::{container, row};
use iced::{Element, Task, Length, Padding, Background};

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(NavSection),
    ToggleDrawer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavSection {
    Agent, Files, Kanban, Cron, Tools, Learn, Channels, Usage, Debug, Settings, Theme, About,
}

pub struct State {
    pub active_section: NavSection,
    pub drawer_open: bool,
}

// ============================================================
// Iced Application functions
// ============================================================

pub fn new() -> (State, Task<Message>) {
    (State { active_section: NavSection::Agent, drawer_open: false }, Task::none())
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::NavigateTo(section) => {
            state.active_section = section;
            state.drawer_open = matches!(section, NavSection::Files | NavSection::Debug | NavSection::Settings);
        }
        Message::ToggleDrawer => { state.drawer_open = !state.drawer_open; }
    }
    Task::none()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let content = content_row_view(&state.active_section, state.drawer_open);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::BG_BACKBOARD)),
            ..Default::default()
        })
        .into()
}

// ============================================================
// Layout assembly — 1+3 layout
// ============================================================

fn content_row_view<'a>(active_section: &'a NavSection, drawer_open: bool) -> Element<'a, Message> {
    let icon_bar = icon_bar::view(active_section);

    let left_panel_opt: Option<Element<'a, Message>> = if *active_section != NavSection::Agent {
        Some(left_panel::view(active_section))
    } else {
        None
    };

    let center_card = chat_area::view(active_section);
    let right_card = drawer::view(drawer_open, active_section);

    let cards_row = if let Some(c1) = left_panel_opt {
        row![c1, center_card, right_card].spacing(theme::CARD_GAP)
    } else {
        row![center_card, right_card].spacing(theme::CARD_GAP)
    };

    row![icon_bar, cards_row]
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding::new(0.0).right(theme::CARD_GAP).bottom(theme::CARD_GAP))
        .into()
}
