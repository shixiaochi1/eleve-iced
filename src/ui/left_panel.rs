use iced::widget::{column, container, text};
use iced::{Element, Length};

use crate::ui::{Message, NavSection, theme};

pub fn view<'a>(active_section: &'a NavSection) -> Element<'a, Message> {
    let name = panel_title(active_section);
    let header = container(text(name).size(14).color(theme::TEXT_PRIMARY))
        .padding([12, 16]);
    let body = container(text("面板内容 - 待实现").size(12).color(theme::TEXT_MUTED))
        .padding([4, 16]);

    container(column![header, body])
        .width(Length::Fixed(theme::LEFT_PANEL_WIDTH))
        .height(Length::Fill)
        .style(theme::card_style())
        .into()
}

fn panel_title(section: &NavSection) -> &'static str {
    match section {
        NavSection::Files => "文件浏览器",
        NavSection::Kanban => "看板",
        NavSection::Cron => "定时任务",
        NavSection::Tools => "工具",
        NavSection::Learn => "学习",
        NavSection::Channels => "频道",
        NavSection::Usage => "用量分析",
        NavSection::Debug => "调试",
        NavSection::Settings => "设置",
        NavSection::Theme => "主题",
        NavSection::About => "关于",
        NavSection::Agent => "",
    }
}
