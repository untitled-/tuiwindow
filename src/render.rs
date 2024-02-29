use std::any::Any;

use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    api::Menu,
    core::{InputEvent, RenderComponent},
};

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

pub trait Render: AsAny {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect);
    fn into_component(self) -> RenderComponent
    where
        Self: Sized + 'static,
    {
        RenderComponent::new(self)
    }

    fn get_menu(&self) -> Option<Menu> {
        None
    }
}

pub trait FocusableRender: Render {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect);

    #[allow(unused_variables)]
    fn render_footer(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {}

    fn get_menu(&self) -> Option<Menu> {
        None
    }
}

impl<T: FocusableRender> Render for T {
    fn render(&mut self, render_props: &RenderProps, buff: &mut Buffer, area: Rect) {
        FocusableRender::render(self, render_props, buff, area)
    }

    fn into_component(self) -> RenderComponent
    where
        Self: Sized + 'static,
    {
        RenderComponent::new_focusable(self)
    }

    fn get_menu(&self) -> Option<Menu> {
        FocusableRender::get_menu(self)
    }
}

impl<T: Render + 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T: Render + 'static> From<T> for RenderComponent {
    fn from(value: T) -> Self {
        value.into_component()
    }
}

pub trait RenderFactory {
    fn render(&mut self) -> RenderComponent;
}

#[derive(Debug)]
pub struct RenderProps {
    pub is_focused: bool,
    pub event: Option<InputEvent>,
    pub event_buffer: Vec<InputEvent>,
}
