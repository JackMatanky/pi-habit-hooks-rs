pub mod guide;
pub mod severity;

pub use guide::GuideResolver;
pub use severity::{evaluate_and_render, MapperResult};
