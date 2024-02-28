#[macro_export]
macro_rules! column_widget {
    ($($obj:expr),*$(,)?) => {
        RenderComponent::column(vec![
        $(
           $obj.into(),
        )*

        ])
    };
}

#[macro_export]
macro_rules! row_widget {
    ($($obj:expr),*$(,)?) => {
        RenderComponent::row(vec![
        $(
           $obj.into(),
        )*

        ])
    };
}

pub use column_widget;
pub use row_widget;
