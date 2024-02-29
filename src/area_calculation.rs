use crate::{api::RenderId, core::RenderComponent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::{collections::VecDeque, rc::Rc};

pub(crate) type AreaCalculator = Rc<dyn Fn(Rect) -> Rect>;

pub(crate) type AreaCalculatorById = Vec<(RenderId, AreaCalculator)>;
pub struct UnrolledComponents(pub(crate) AreaCalculatorById);

pub fn unroll(c: &RenderComponent) -> UnrolledComponents {
    let mut ops = Vec::new();

    let mut queue: VecDeque<(&RenderComponent, AreaCalculator)> = VecDeque::new();

    queue.push_back((c, Rc::new(|area: Rect| area)));

    while let Some((c, a)) = queue.pop_front() {
        match c {
            RenderComponent::Layout(_, direction, children) => {
                let layout = Rc::new(Layout::new(
                    match direction {
                        crate::core::LayoutDirection::Column => Direction::Horizontal,
                        crate::core::LayoutDirection::Row => Direction::Vertical,
                    },
                    children.iter().map(|_| Constraint::Fill(1)),
                ));

                for (i, c) in children.iter().enumerate() {
                    let ac = Rc::clone(&a);
                    let lc = Rc::clone(&layout);
                    queue.push_back((c, Rc::new(move |area: Rect| lc.split(ac(area))[i])));
                }
            }
            RenderComponent::Render(details) => ops.push((details.id, a)),
            RenderComponent::Factory(_) => todo!(),
        }
    }

    UnrolledComponents(ops)
}
