use iced::widget::{button, column, container, rule, Svg, Space};
use iced::{Element, Background, Length, Border, Gradient, Padding};
use std::path::PathBuf;

use crate::ui::{Message, NavSection};
use super::theme;

pub fn view<'a>(active_section: &'a NavSection) -> Element<'a, Message> {
    let logo_btn = logo_button();
    let nav_items = column(nav_buttons(active_section)).spacing(2);

    let bottom_items = column![
        nav_icon("settings", NavSection::Settings, active_section),
        nav_icon("palette", NavSection::Theme, active_section),
        nav_icon("info", NavSection::About, active_section),
    ]
    .spacing(2);

    let content = column![
        logo_btn,
        nav_items,
        Space::new().height(Length::Fill),
        rule::horizontal(1.0).style(|_: &iced::Theme| rule::Style {
            color: theme::SEPARATOR,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        }),
        bottom_items,
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center);

    let icon_padding = (theme::ICON_BAR_WIDTH - theme::ICON_BTN_SIZE) / 2.0;
    container(content)
        .width(Length::Fixed(theme::ICON_BAR_WIDTH))
        .height(Length::Fill)
        .padding(Padding::new(0.0).bottom(theme::ICON_BAR_PADDING).left(icon_padding).right(icon_padding))
        .style(|_: &iced::Theme| container::Style { background: None, ..Default::default() })
        .into()
}

fn logo_button<'a>() -> iced::widget::Button<'a, Message, iced::Theme, iced::Renderer> {
    button(
        Svg::from_path(PathBuf::from("assets/icons/Elogo.svg"))
            .width(Length::Fixed(theme::LOGO_ICON_SIZE))
            .height(Length::Fixed(theme::LOGO_ICON_SIZE)),
    )
    .width(Length::Fixed(theme::ICON_BTN_SIZE))
    .height(Length::Fixed(theme::ICON_BTN_SIZE))
    .padding(0)
    .style(|_theme: &iced::Theme, status| {
        let is_active = matches!(status, iced::widget::button::Status::Hovered);
        iced::widget::button::Style {
            background: if is_active {
                Some(Background::Color(theme::BG_HOVER))
            } else {
                None
            },
            border: Border { radius: 10.0.into(), ..Default::default() },
            ..Default::default()
        }
    })
    .on_press(Message::NavigateTo(NavSection::Agent))
}

fn nav_buttons<'a>(active_section: &'a NavSection) -> Vec<Element<'a, Message>> {
    vec![
        nav_icon("folder-git-2", NavSection::Files, active_section),
        nav_icon("layout-grid", NavSection::Kanban, active_section),
        nav_icon("clock", NavSection::Cron, active_section),
        nav_icon("wrench", NavSection::Tools, active_section),
        nav_icon("book-open", NavSection::Learn, active_section),
        nav_icon("radio", NavSection::Channels, active_section),
        nav_icon("chart-column", NavSection::Usage, active_section),
        nav_icon("bug", NavSection::Debug, active_section),
    ]
}

fn nav_icon<'a>(
    icon_name: &'static str,
    section: NavSection,
    active_section: &'a NavSection,
) -> Element<'a, Message> {
    let is_active = section == *active_section;
    let icon_color = if is_active { theme::TEXT_ON_ACCENT } else { theme::TEXT_MUTED };

    button(
        Svg::from_path(icon_path(icon_name))
            .width(Length::Fixed(theme::ICON_SIZE))
            .height(Length::Fixed(theme::ICON_SIZE))
            .style(move |_: &iced::Theme, _| iced::widget::svg::Style { color: Some(icon_color) }),
    )
    .width(Length::Fixed(theme::ICON_BTN_SIZE))
    .height(Length::Fixed(theme::ICON_BTN_SIZE))
    .padding(theme::NAV_ICON_PADDING)
    .style(move |_: &iced::Theme, status| {
        let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
        if is_active {
            iced::widget::button::Style {
                background: Some(Background::Gradient(Gradient::Linear(
                    iced::gradient::Linear::new(iced::Degrees(180.0))
                        .add_stop(0.0, theme::ACCENT)
                        .add_stop(1.0, theme::ACCENT_HOVER),
                ))),
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        } else if is_hovered {
            iced::widget::button::Style {
                background: Some(Background::Color(theme::BG_HOVER)),
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        } else {
            iced::widget::button::Style {
                border: Border { radius: 10.0.into(), ..Default::default() },
                ..Default::default()
            }
        }
    })
    .on_press(Message::NavigateTo(section))
    .into()
}

fn icon_path(name: &str) -> PathBuf {
    PathBuf::from("assets/icons").join(format!("{}.svg", name))
}
