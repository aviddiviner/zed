use gpui::Context;
use workspace::Workspace;

/// In Aleph (always-local), there are no remote disconnections to handle.
/// This module is kept as a stub for compatibility.
pub struct DisconnectedOverlay;

impl DisconnectedOverlay {
    pub fn register(
        _workspace: &mut Workspace,
        _window: Option<&mut gpui::Window>,
        _cx: &mut Context<Workspace>,
    ) {
    }
}
