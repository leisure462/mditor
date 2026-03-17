use gpui::{AnyElement, Empty, IntoElement, Window};
use ui::Context;

use crate::TitleBar;

impl TitleBar {
    pub(crate) fn render_collaborator_list(
        &self,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> impl IntoElement {
        Empty
    }

    pub(crate) fn render_call_controls(
        &self,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        Vec::new()
    }
}
