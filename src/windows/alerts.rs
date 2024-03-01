use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::render::RenderTimed;

#[derive(Debug)]
pub struct Alert {
    title: String,
    message: String,
    duration: Duration,
    rendered_time: Option<Instant>,
}

impl Alert {
    pub fn new<T1: Into<String>, T2: Into<String>>(
        title: T1,
        message: T2,
        duration: Duration,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            duration,
            rendered_time: None,
        }
    }
}

impl Alert {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let time = self.rendered_time.get_or_insert(Instant::now());

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        Clear.render(popup_area, buf);

        let layout = Layout::new(
            Direction::Vertical,
            vec![Constraint::Fill(1), Constraint::Length(2)],
        )
        .split(popup_area);
        Paragraph::new(self.message.as_str())
            .wrap(Wrap { trim: true })
            .block(
                Block::new()
                    .borders(Borders::all())
                    .style(Style::new().fg(Color::Black).bg(Color::White))
                    .title(self.title.as_str()),
            )
            .render(layout[0], buf);

        Text::raw(format!(
            "{}/{}",
            (self.duration - time.elapsed()).as_secs(),
            self.duration.as_secs()
        ))
        .render(layout[1], buf);
    }
}

impl RenderTimed for Alert {
    fn is_visible(&self) -> bool {
        self.rendered_time
            .map_or(true, |t| t.elapsed() < self.duration)
    }
}

#[derive(Default, Debug)]
pub struct AlertManager {
    alerts: Vec<Alert>,
}

impl AlertManager {
    pub fn first_visible(&mut self) -> Option<&mut Alert> {
        self.alerts.iter_mut().find(|a| a.is_visible())
    }

    pub fn schedule(&mut self, alert: Alert) {
        self.alerts.push(alert)
    }

    pub fn alert<T: Into<String>>(&mut self, msg: T) {
        self.alerts
            .push(Alert::new("Alert", msg, Duration::from_secs(1)))
    }
}
