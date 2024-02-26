use std::collections::HashMap;

use ratatui::layout::{Constraint, Direction, Rect};
use uuid::Uuid;
pub enum LayoutDirection {
    Column,
    Row,
}

pub trait Render {
    fn render(&mut self, render_props: &RenderProps, area: Rect);
}

type RenderFn = dyn Render;

pub enum Component {
    Layout(uuid::Uuid, LayoutDirection, Vec<Component>),
    Render {
        id: uuid::Uuid,
        focusable: bool,
        render: Box<RenderFn>,
    },
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Key(char),
}

#[derive(Debug)]
pub struct RenderProps {
    pub is_focused: bool,
    pub event: Option<InputEvent>,
    pub event_buffer: Vec<InputEvent>,
}

pub struct VRenderProps<'a> {
    focused_element: Option<&'a Uuid>,
    event: Option<InputEvent>,
}

impl Component {
    pub fn new<T: Render + 'static>(render_fn: T) -> Self {
        Self::Render {
            id: Uuid::new_v4(),
            focusable: false,
            render: Box::new(render_fn),
        }
    }

    pub fn new_focusable<T: Render + 'static>(render_fn: T) -> Self {
        Self::Render {
            id: Uuid::new_v4(),
            focusable: true,
            render: Box::new(render_fn),
        }
    }

    pub fn is_focusable(&self) -> bool {
        match self {
            Component::Layout(_, _, _) => false,
            Component::Render { focusable, .. } => *focusable,
        }
    }

    pub fn column(children: Vec<Component>) -> Self {
        Component::Layout(uuid::Uuid::new_v4(), LayoutDirection::Column, children)
    }

    pub fn row(children: Vec<Component>) -> Self {
        Component::Layout(uuid::Uuid::new_v4(), LayoutDirection::Row, children)
    }

    pub fn flatten_ids(&self) -> Vec<Uuid> {
        match self {
            Component::Layout(_, _, children) => {
                children.iter().flat_map(|c| c.flatten_ids()).collect()
            }
            Component::Render { id, focusable, .. } => {
                if *focusable {
                    vec![*id]
                } else {
                    vec![]
                }
            }
        }
    }

    pub fn visit(
        &mut self,
        opts: &VRenderProps,
        component_buffer: &mut ComponentBuffer,
        area: Rect,
    ) {
        match self {
            Component::Layout(_, layout, children) => {
                let layout = ratatui::layout::Layout::new(
                    match layout {
                        LayoutDirection::Column => Direction::Vertical,
                        LayoutDirection::Row => Direction::Horizontal,
                    },
                    children.iter().map(|_| Constraint::Fill(1)),
                )
                .split(area);

                for (a, c) in layout.iter().zip(children.iter_mut()) {
                    c.visit(opts, component_buffer, *a);
                }
            }
            Component::Render { id, render, .. } => {
                let is_focused = opts.focused_element.map(|fid| fid == id).unwrap_or(false);
                render.render(
                    &RenderProps {
                        is_focused,
                        event: if is_focused { opts.event.clone() } else { None },
                        event_buffer: component_buffer.get_buffer(id),
                    },
                    area,
                )
            }
        }
    }
}

#[derive(Default)]
pub struct ComponentBuffer {
    buff: HashMap<Uuid, Vec<InputEvent>>,
}

impl ComponentBuffer {
    pub fn add_event(&mut self, id: Uuid, event: &Option<InputEvent>) {
        if let Some(ev) = event {
            self.buff.entry(id).or_default().push(ev.clone());
        }
    }

    pub fn get_buffer(&self, id: &Uuid) -> Vec<InputEvent> {
        self.buff.get(id).unwrap_or(&vec![]).clone()
    }
}

#[cfg(test)]
mod tests {

    use crate::{ComponentBuffer, InputEvent, Render, VRenderProps};

    use super::Component;

    struct TestRender {
        name: String,
        text_content: String,
    }

    impl TestRender {
        fn new<T: Into<String>>(s: T) -> Self {
            Self {
                name: s.into(),
                text_content: String::new(),
            }
        }
    }

    impl Render for TestRender {
        fn render(&mut self, render_props: &crate::RenderProps, _area: ratatui::prelude::Rect) {
            if let Some(ev) = &render_props.event {
                match ev {
                    InputEvent::Key(k) => self.text_content.push(*k),
                }
            }
            println!("{}:{}", self.name, self.text_content)
        }
    }

    #[test]
    fn it_works() {
        let mut app = Component::row(vec![
            Component::new(TestRender::new("c1")),
            Component::new_focusable(TestRender::new("c2")),
            Component::column(vec![
                Component::new_focusable(TestRender::new("c3")),
                Component::new_focusable(TestRender::new("c4")),
            ]),
        ]);

        let mut event_buffer = ComponentBuffer::default();
        let focusable_elements = app.flatten_ids();
        let mut focus_iterator = focusable_elements.iter().cycle();

        focus_iterator.next();

        let focused_element = focus_iterator.next();

        println!("Loop 1\n\n");
        event_buffer.add_event(*focused_element.unwrap(), &None);
        app.visit(
            &VRenderProps {
                focused_element,
                event: None,
            },
            &mut event_buffer,
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        );

        println!("Loop 2\n\n");
        let event = Some(InputEvent::Key('k'));
        event_buffer.add_event(*focused_element.unwrap(), &event);

        app.visit(
            &VRenderProps {
                focused_element,
                event,
            },
            &mut event_buffer,
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        );

        println!("Loop 3\n\n");

        let event = Some(InputEvent::Key('k'));
        event_buffer.add_event(*focused_element.unwrap(), &event);
        app.visit(
            &VRenderProps {
                focused_element,
                event,
            },
            &mut event_buffer,
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        );
        println!("Loop 4");
        let focused_element = focus_iterator.next();
        let event = Some(InputEvent::Key('k'));
        event_buffer.add_event(*focused_element.unwrap(), &event);
        app.visit(
            &VRenderProps {
                focused_element,
                event,
            },
            &mut event_buffer,
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        )
    }
}
