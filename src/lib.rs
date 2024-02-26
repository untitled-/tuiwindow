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

pub struct VRenderProps {
    focused_element: Option<Uuid>,
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

    pub fn render(
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
                    c.render(opts, component_buffer, *a);
                }
            }
            Component::Render { id, render, .. } => {
                let is_focused = opts.focused_element.map(|fid| fid == *id).unwrap_or(false);
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

pub trait LoopManager<'a>: Iterator<Item = (Option<InputEvent>, Option<Uuid>)> {}

#[cfg(test)]
mod tests {

    use uuid::Uuid;

    use crate::{ComponentBuffer, InputEvent, LoopManager, Render, VRenderProps};

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
            println!(
                "{}:{} (focused?:{})",
                self.name, self.text_content, render_props.is_focused
            )
        }
    }

    struct TestLoopManager {
        focusable_elements: Vec<Option<Uuid>>,
        current_element: usize,
        current_event: usize,
        events: Vec<Option<InputEvent>>,
    }

    impl TestLoopManager {
        fn new(focusable_elements: Vec<Option<Uuid>>, events: Vec<Option<InputEvent>>) -> Self {
            Self {
                focusable_elements,
                current_element: 0,
                current_event: 0,
                events,
            }
        }
    }

    impl Iterator for TestLoopManager {
        type Item = (Option<InputEvent>, Option<Uuid>);

        fn next(&mut self) -> Option<Self::Item> {
            let next = self
                .focusable_elements
                .get(self.current_element)
                .cloned()
                .flatten();
            if self.current_element + 1 == self.focusable_elements.len() {
                self.current_element = 0;
            } else {
                self.current_element += 1;
            }

            let next_event = self.events.get(self.current_event).cloned().flatten();
            if self.current_event + 1 >= self.events.len() {
                self.current_event = 0;
            } else {
                self.current_event += 1;
            }

            Some((next_event, next))
        }
    }

    impl<'a> LoopManager<'a> for TestLoopManager {}

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

        let all_focusable_elements = app.flatten_ids();

        let mut all_focusable_iter = all_focusable_elements.iter().cycle();
        all_focusable_iter.next();

        let f1 = all_focusable_iter.next().cloned();
        let f2 = all_focusable_iter.next().cloned();

        let mut test_loop_manager = TestLoopManager::new(
            vec![f1, f1, f1, f2, f2, f2],
            vec![
                None,
                Some(InputEvent::Key('a')),
                Some(InputEvent::Key('b')),
                Some(InputEvent::Key('c')),
            ],
        );

        let mut event_buffer = ComponentBuffer::default();

        for i in 0..5 {
            let (event, focused_element) = test_loop_manager.next().unwrap();

            println!(
                "\n\nLoop {}: ev:{:?}, focused:{:?}",
                i, event, focused_element
            );
            event_buffer.add_event(focused_element.unwrap(), &event);
            app.render(
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
        }
    }
}
