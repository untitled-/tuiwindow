use std::{
    any::Any,
    cell::{RefCell, RefMut},
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
};
use uuid::Uuid;

use crate::api::Menu;

#[derive(PartialEq, Eq)]
pub enum LayoutDirection {
    Column,
    Row,
}

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

/// Wrapper for factory of components.
/// We use interior mutability to cache the results of the render factory.
pub struct RenderFactoryBox {
    cached: RefCell<Option<RenderComponent>>,
    render: RefCell<Box<dyn RenderFactory>>,
}

struct RW<'a> {
    inner: RefMut<'a, RenderComponent>,
}

impl<'b> Deref for RW<'b> {
    type Target = RenderComponent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct RWMut<'a> {
    inner: RefMut<'a, RenderComponent>,
}

impl<'b> Deref for RWMut<'b> {
    type Target = RenderComponent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'b> DerefMut for RWMut<'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl RenderFactoryBox {
    fn build_component(&self) -> RenderComponent {
        self.render.borrow_mut().render()
    }

    fn cache(&self) -> RefMut<'_, RenderComponent> {
        RefMut::map(self.cached.borrow_mut(), |c| {
            c.get_or_insert_with(|| self.build_component())
        })
    }

    fn component(&self) -> RW<'_> {
        RW {
            inner: self.cache(),
        }
    }

    fn component_mut(&mut self) -> RWMut<'_> {
        RWMut {
            inner: self.cache(),
        }
    }
}

pub trait RenderFlow {
    fn render(
        &mut self,
        opts: &VRenderProps,
        component_buffer: &mut ComponentBuffer,
        buff: &mut Buffer,
        area: Rect,
    );

    fn get_focusable_elements(&self) -> Vec<Uuid>;

    fn get_menu(&self) -> Option<Menu> {
        None
    }
}

pub struct RenderComponentDetails {
    pub id: uuid::Uuid,
    pub focusable: bool,
    pub render: Box<dyn Render>,
}

pub enum RenderComponent {
    Layout(uuid::Uuid, LayoutDirection, Vec<RenderComponent>),
    Render(RenderComponentDetails),
    Factory(Box<RenderFactoryBox>),
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Key(char),
    FocusWindow,
    FocusNext,
    FocusPrevious,
}

#[derive(Debug)]
pub struct RenderProps {
    pub is_focused: bool,
    pub event: Option<InputEvent>,
    pub event_buffer: Vec<InputEvent>,
}

pub struct VRenderProps {
    pub focused_element: Option<Uuid>,
    pub event: Option<InputEvent>,
}

impl RenderComponent {
    pub fn new<T: Render + 'static>(render_fn: T) -> Self {
        Self::Render(RenderComponentDetails {
            id: Uuid::new_v4(),
            focusable: false,
            render: Box::new(render_fn),
        })
    }

    pub fn new_focusable<T: Render + 'static>(render_fn: T) -> Self {
        Self::Render(RenderComponentDetails {
            id: Uuid::new_v4(),
            focusable: true,
            render: Box::new(render_fn),
        })
    }

    pub fn new_factory<T: RenderFactory + 'static>(render_factory: T) -> Self {
        Self::Factory(Box::new(RenderFactoryBox {
            cached: RefCell::new(None),
            render: RefCell::new(Box::new(render_factory)),
        }))
    }

    pub fn is_focusable(&self) -> bool {
        match self {
            RenderComponent::Layout(_, _, _) => false,
            RenderComponent::Render(details) => details.focusable,
            RenderComponent::Factory(_) => false,
        }
    }

    pub fn column(children: Vec<RenderComponent>) -> Self {
        RenderComponent::Layout(uuid::Uuid::new_v4(), LayoutDirection::Column, children)
    }

    pub fn row(children: Vec<RenderComponent>) -> Self {
        RenderComponent::Layout(uuid::Uuid::new_v4(), LayoutDirection::Row, children)
    }

    pub fn flatten_ids(&self) -> Vec<Uuid> {
        match self {
            RenderComponent::Layout(_, _, children) => {
                children.iter().flat_map(|c| c.flatten_ids()).collect()
            }
            RenderComponent::Render(details) => {
                if details.focusable {
                    vec![details.id]
                } else {
                    vec![]
                }
            }
            RenderComponent::Factory(factory) => factory.component().flatten_ids(),
        }
    }

    pub fn visit_with_downcast<T: Render + Any>(&self, f: &mut dyn FnMut(Option<&T>)) {
        match self {
            RenderComponent::Layout(_, _, children) => {
                for c in children {
                    c.visit_with_downcast(f)
                }
            }
            RenderComponent::Render(details) => {
                f(details.render.as_any().downcast_ref::<T>());
            }
            RenderComponent::Factory(factory) => factory.component().visit_with_downcast(f),
        };
    }
    pub fn visit(&self, f: &mut dyn FnMut(&RenderComponentDetails) -> bool) -> bool {
        match self {
            RenderComponent::Layout(_, _, children) => {
                for c in children {
                    if !c.visit(f) {
                        return false;
                    }
                }
                true
            }
            RenderComponent::Render(details) => f(details),
            RenderComponent::Factory(factory) => factory.component().visit(f),
        }
    }
}

impl RenderFlow for RenderComponent {
    fn render(
        &mut self,
        opts: &VRenderProps,
        component_buffer: &mut ComponentBuffer,
        buff: &mut Buffer,
        area: Rect,
    ) {
        match self {
            RenderComponent::Layout(_, layout, children) => {
                let layout = ratatui::layout::Layout::new(
                    match layout {
                        LayoutDirection::Column => Direction::Vertical,
                        LayoutDirection::Row => Direction::Horizontal,
                    },
                    children.iter().map(|_| Constraint::Fill(1)),
                )
                .split(area);

                for (a, c) in layout.iter().zip(children.iter_mut()) {
                    c.render(opts, component_buffer, buff, *a);
                }
            }
            RenderComponent::Render(details) => {
                let is_focused = opts
                    .focused_element
                    .map(|fid| fid == details.id)
                    .unwrap_or(false);
                details.render.render(
                    &RenderProps {
                        is_focused,
                        event: if is_focused { opts.event.clone() } else { None },
                        event_buffer: component_buffer.get_buffer(&details.id),
                    },
                    buff,
                    area,
                )
            }
            // Component::Factory(factory) => factory.render(opts, component_buffer, area),
            RenderComponent::Factory(factory) => {
                factory
                    .component_mut()
                    .render(opts, component_buffer, buff, area)
            }
        }
    }

    fn get_focusable_elements(&self) -> Vec<Uuid> {
        match self {
            RenderComponent::Layout(_, _, children) => {
                children.iter().flat_map(|c| c.flatten_ids()).collect()
            }
            RenderComponent::Render(details) => {
                if details.focusable {
                    vec![details.id]
                } else {
                    vec![]
                }
            }
            RenderComponent::Factory(factory) => factory.component().flatten_ids(),
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

pub trait Greet {
    fn greet(&self) -> String;
}

#[cfg(test)]
mod tests {

    extern crate macros;

    use ratatui::buffer::Buffer;
    use uuid::Uuid;

    use crate::core::RenderFlow;
    use crate::macros::column_widget;
    use crate::row_widget;

    use super::{
        ComponentBuffer, InputEvent, LoopManager, Render, RenderFactory, RenderProps, VRenderProps,
    };

    use super::RenderComponent;

    #[derive(Debug)]
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
        fn render(
            &mut self,
            render_props: &RenderProps,
            _buff: &mut Buffer,
            _area: ratatui::prelude::Rect,
        ) {
            if let Some(InputEvent::Key(k)) = &render_props.event {
                self.text_content.push(*k)
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
        let mut app = RenderComponent::row(vec![
            TestRender::new("c1").into(),
            RenderComponent::new_focusable(TestRender::new("c2")),
            RenderComponent::column(vec![
                RenderComponent::new_focusable(TestRender::new("c3")),
                RenderComponent::new_focusable(TestRender::new("c4")),
            ]),
        ]);

        run_loop(&mut app);
        app.visit_with_downcast::<TestRender>(&mut |x| {
            assert!(x.is_some());

            let node = x.unwrap();

            if node.name == "c3" {
                assert_eq!(node.text_content, String::from("ab"));
            }
        });
    }

    struct TestFactory {}

    impl RenderFactory for TestFactory {
        fn render(&mut self) -> RenderComponent {
            RenderComponent::column(vec![
                RenderComponent::new_focusable(TestRender::new("c3")),
                RenderComponent::new_focusable(TestRender::new("c4")),
            ])
        }
    }

    #[test]
    fn it_works_with_factory() {
        let mut app = RenderComponent::row(vec![
            TestRender::new("c1").into(),
            RenderComponent::new_focusable(TestRender::new("c2")),
            RenderComponent::new_factory(TestFactory {}),
        ]);

        run_loop(&mut app);
        app.visit_with_downcast::<TestRender>(&mut |x| {
            assert!(x.is_some());

            let node = x.unwrap();

            if node.name == "c3" {
                assert_eq!(node.text_content, String::from("ab"));
            }
        });
    }

    fn run_loop(app: &mut RenderComponent) {
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

        let area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        let mut buffer = Buffer::empty(area);
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
                &mut buffer,
                area,
            );
        }
    }
    #[test]
    fn test_macros() {
        let mut app = column_widget!(
            TestRender::new("c1"),
            RenderComponent::new_focusable(TestRender::new("c2")),
            RenderComponent::new_factory(TestFactory {}),
        );

        run_loop(&mut app);
        app.visit_with_downcast::<TestRender>(&mut |x| {
            assert!(x.is_some());

            let node = x.unwrap();

            if node.name == "c3" {
                assert_eq!(node.text_content, String::from("ab"));
            }
        });
    }

    #[test]
    fn test_visit() {
        let app = column_widget!(
            TestRender::new("c1"),
            TestRender::new("c2"),
            row_widget!(TestRender::new("c3"), TestRender::new("c4"))
        );

        let mut visited = vec![];

        app.visit(&mut |rc| {
            visited.push(rc.id);
            true
        });

        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_visit_with_factory() {
        let app = column_widget!(
            TestRender::new("c1"),
            RenderComponent::new_focusable(TestRender::new("c2")),
            RenderComponent::new_factory(TestFactory {}),
        );

        let mut visited = vec![];

        app.visit(&mut |details| {
            visited.push(details.id);
            true
        });

        visited.sort();

        assert_eq!(visited.len(), 4);
    }
}
