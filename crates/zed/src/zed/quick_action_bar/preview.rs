use editor::Editor;
use gpui::{Action, AnyElement, Modifiers, WeakEntity};
use markdown_preview::{
    OpenPreviewToTheSide as MarkdownOpenPreviewToTheSide,
    markdown_preview_view::MarkdownPreviewView,
};
use ui::{Tooltip, prelude::*, text_for_keystroke};
use workspace::Workspace;

use crate::zed::set_markdown_source_mode;

use super::QuickActionBar;

impl QuickActionBar {
    pub fn render_preview_button(
        &self,
        workspace_handle: WeakEntity<Workspace>,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let is_preview_active = self
            .active_item
            .as_deref()
            .and_then(|item| item.downcast::<MarkdownPreviewView>())
            .is_some();
        let has_markdown_editor = if let Some(workspace) = self.workspace.upgrade() {
            workspace.update(cx, |workspace, cx| {
                MarkdownPreviewView::resolve_active_item_as_markdown_editor(workspace, cx).is_some()
            })
        } else {
            false
        };
        has_markdown_editor.then_some(())?;

        let alt_click = gpui::Keystroke {
            key: "click".into(),
            modifiers: Modifiers::alt(),
            ..Default::default()
        };

        let button = IconButton::new("toggle-markdown-preview", IconName::Eye)
            .icon_size(IconSize::Small)
            .style(ButtonStyle::Subtle)
            .toggle_state(is_preview_active)
            .tooltip(move |window, cx| {
                if is_preview_active {
                    Tooltip::text("切换到源码模式")(window, cx)
                } else {
                    Tooltip::with_meta(
                        "预览 Markdown",
                        Some(&markdown_preview::OpenPreview as &dyn gpui::Action),
                        format!(
                            "{} 可在分栏中打开",
                            text_for_keystroke(&alt_click.modifiers, &alt_click.key, cx)
                        ),
                        cx,
                    )
                }
            })
            .on_click(move |_, window, cx| {
                if let Some(workspace) = workspace_handle.upgrade() {
                    workspace.update(cx, |workspace, cx| {
                        if let Some(preview) = workspace
                            .active_item(cx)
                            .and_then(|item| item.act_as::<MarkdownPreviewView>(cx))
                        {
                            let Some(editor) = preview.read(cx).linked_editor() else {
                                return;
                            };
                            set_markdown_source_mode(editor.entity_id(), true, cx);
                            let preview_item_id = preview.entity_id();
                            workspace.active_pane().update(cx, |pane, cx| {
                                pane.add_item(Box::new(editor), true, true, None, window, cx);
                                pane.remove_item(preview_item_id, false, true, window, cx);
                            });
                            return;
                        }

                        if window.modifiers().alt {
                            window.dispatch_action(MarkdownOpenPreviewToTheSide.boxed_clone(), cx);
                            return;
                        }

                        let Some(active_item) = workspace.active_item(cx) else {
                            return;
                        };
                        let Some(editor) = active_item.act_as::<Editor>(cx) else {
                            return;
                        };
                        set_markdown_source_mode(editor.entity_id(), false, cx);
                        let editor_item_id = active_item.item_id();
                        MarkdownPreviewView::open_preview_for_editor(workspace, editor, window, cx);
                        workspace.active_pane().update(cx, |pane, cx| {
                            pane.remove_item(editor_item_id, false, true, window, cx);
                        });
                    });
                }
            });

        Some(button.into_any_element())
    }
}
