pub mod context;
pub mod model;

use context::MockAppBuilder;

pub fn build_mock_app() -> MockAppBuilder {
    MockAppBuilder::new()
}
