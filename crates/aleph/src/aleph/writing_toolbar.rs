use editor::Editor;
use gpui::{Context, Entity, EventEmitter, Render, WeakEntity, Window};
use markdown_preview::{
    OpenPreview, OpenPreviewToTheSide, markdown_preview_view::MarkdownPreviewView,
};
use ui::prelude::*;
use ui::{IconButton, IconSize, Tooltip, text_for_keystroke};
use workspace::{ItemHandle, ToolbarItemEvent, ToolbarItemLocation, ToolbarItemView, Workspace};

pub struct WritingToolbar {
    workspace: WeakEntity<Workspace>,
    active_item: Option<Box<dyn ItemHandle>>,
}

impl WritingToolbar {
    pub fn new(workspace: &Workspace) -> Self {
        Self {
            workspace: workspace.weak_handle(),
            active_item: None,
        }
    }

    fn active_editor(&self) -> Option<Entity<Editor>> {
        self.active_item
            .as_ref()
            .and_then(|item| item.downcast::<Editor>())
    }

    fn render_preview_button(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let workspace = self.workspace.upgrade()?;
        let is_markdown = workspace.update(cx, |workspace, cx| {
            MarkdownPreviewView::resolve_active_item_as_markdown_editor(workspace, cx).is_some()
        });

        if !is_markdown {
            return None;
        }

        let workspace_handle = self.workspace.clone();
        let alt_click = gpui::Keystroke {
            key: "click".into(),
            modifiers: gpui::Modifiers::alt(),
            ..Default::default()
        };

        Some(
            IconButton::new("toggle-markdown-preview", IconName::Eye)
                .icon_size(IconSize::Small)
                .style(ButtonStyle::Subtle)
                .tooltip(move |_window, cx| {
                    Tooltip::with_meta(
                        "Preview Markdown",
                        Some(&OpenPreview),
                        format!(
                            "{} to open in a split",
                            text_for_keystroke(&alt_click.modifiers, &alt_click.key, cx)
                        ),
                        cx,
                    )
                })
                .on_click(move |_, window, cx| {
                    if let Some(workspace) = workspace_handle.upgrade() {
                        workspace.update(cx, |_, cx| {
                            if window.modifiers().alt {
                                window.dispatch_action(Box::new(OpenPreviewToTheSide), cx);
                            } else {
                                window.dispatch_action(Box::new(OpenPreview), cx);
                            }
                        });
                    }
                }),
        )
    }
}

impl Render for WritingToolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .id("writing-toolbar")
            .gap(DynamicSpacing::Base01.rems(cx))
            .children(self.render_preview_button(cx))
    }
}

impl EventEmitter<ToolbarItemEvent> for WritingToolbar {}

impl ToolbarItemView for WritingToolbar {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> ToolbarItemLocation {
        self.active_item = active_pane_item.map(ItemHandle::boxed_clone);
        if self.active_editor().is_some() {
            ToolbarItemLocation::PrimaryRight
        } else {
            ToolbarItemLocation::Hidden
        }
    }
}
