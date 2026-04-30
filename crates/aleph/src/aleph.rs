use agent_ui::AgentPanel;
use anyhow::Context as _;
use assets::Assets;
use futures::{FutureExt as _, StreamExt, channel::mpsc, select_biased};
use git_ui::git_panel::GitPanel;
use gpui::{
    App, AppContext as _, AsyncWindowContext, Context, Entity, Focusable as _, KeyBinding, Menu,
    MenuItem, OsAction, ReadGlobal as _, Size, Task, TitlebarOptions, UpdateGlobal as _,
    WeakEntity, Window, WindowHandle, WindowKind, WindowOptions, actions, point, px,
};
use image_viewer::ImageInfo;
use language::Capability;
use outline_panel::OutlinePanel;
use project::DisableAiSettings;
use project_panel::ProjectPanel;
use search::project_search::ProjectSearchBar;
use settings::{
    BaseKeymap, DEFAULT_KEYMAP_PATH, KeybindSource, KeymapFile, KeymapFileLoadResult, Settings,
    SettingsStore,
};
use sidebar::Sidebar;
use std::{future::Future, path::PathBuf, sync::Arc};
use theme::ActiveTheme;
use util::{ResultExt, asset_str};
use uuid::Uuid;
use workspace::Pane;
use workspace::{
    AppState, CloseIntent, CloseWindow, MultiWorkspace, NewFile, NewWindow, Panel, Workspace,
    WorkspaceSettings, with_active_or_new_workspace,
};
use zed_actions::{About, OpenSettingsFile, Quit};

mod writing_toolbar;
use writing_toolbar::WritingToolbar;

use fs::Fs;

actions!(
    aleph,
    [Hide, HideOthers, ShowAll, OpenDefaultSettings, OpenLog,]
);

pub fn init(cx: &mut App) {
    #[cfg(target_os = "macos")]
    cx.on_action(|_: &Hide, cx| cx.hide());
    #[cfg(target_os = "macos")]
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    #[cfg(target_os = "macos")]
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(quit);

    cx.on_action(|_: &zed_actions::IncreaseBufferFontSize, cx: &mut App| {
        theme_settings::increase_buffer_font_size(cx);
    });
    cx.on_action(|_: &zed_actions::DecreaseBufferFontSize, cx: &mut App| {
        theme_settings::decrease_buffer_font_size(cx);
    });
    cx.on_action(|_: &zed_actions::ResetBufferFontSize, cx: &mut App| {
        theme_settings::reset_buffer_font_size(cx);
    });

    cx.on_action(|_: &zed_actions::OpenKeymapFile, cx| {
        with_active_or_new_workspace(cx, |_, window, cx| {
            open_settings_file(
                paths::keymap_file(),
                || settings::initial_keymap_content().as_ref().into(),
                window,
                cx,
            );
        });
    })
    .on_action(|_: &OpenSettingsFile, cx| {
        with_active_or_new_workspace(cx, |_, window, cx| {
            open_settings_file(
                paths::settings_file(),
                || settings::initial_user_settings_content().as_ref().into(),
                window,
                cx,
            );
        });
    })
    .on_action(|_: &OpenDefaultSettings, cx| {
        with_active_or_new_workspace(cx, |workspace, window, cx| {
            open_bundled_file(
                workspace,
                settings::default_settings(),
                "Default Settings",
                "JSON",
                window,
                cx,
            );
        });
    })
    .on_action(|_: &zed_actions::OpenDefaultKeymap, cx| {
        with_active_or_new_workspace(cx, |workspace, window, cx| {
            open_bundled_file(
                workspace,
                settings::default_keymap(),
                "Default Key Bindings",
                "JSON",
                window,
                cx,
            );
        });
    })
    .on_action(|_: &zed_actions::OpenLicenses, cx| {
        with_active_or_new_workspace(cx, |workspace, window, cx| {
            open_bundled_file(
                workspace,
                asset_str::<Assets>("licenses.md"),
                "Open Source License Attribution",
                "Markdown",
                window,
                cx,
            );
        });
    });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.spawn(async move |cx| {
        let workspaces: Vec<WindowHandle<MultiWorkspace>> = cx.update(|cx| {
            cx.windows()
                .into_iter()
                .filter_map(|window| window.downcast::<MultiWorkspace>())
                .collect()
        });

        for window in &workspaces {
            if let Some(should_close) = window
                .update(cx, |multi_workspace, window, cx| {
                    multi_workspace.workspace().update(cx, |workspace, cx| {
                        workspace.prepare_to_close(CloseIntent::Quit, window, cx)
                    })
                })
                .log_err()
            {
                if !should_close.await.unwrap_or(true) {
                    return;
                }
            }
        }

        cx.update(|cx| cx.quit());
    })
    .detach();
}

pub fn build_window_options(display_uuid: Option<Uuid>, cx: &mut App) -> WindowOptions {
    let display = display_uuid.and_then(|uuid| {
        cx.displays()
            .into_iter()
            .find(|display| display.uuid().ok() == Some(uuid))
    });
    let app_id = "dev.aleph.Aleph";
    let window_decorations = match std::env::var("ZED_WINDOW_DECORATIONS") {
        Ok(val) if val == "server" => gpui::WindowDecorations::Server,
        Ok(val) if val == "client" => gpui::WindowDecorations::Client,
        _ => match WorkspaceSettings::get_global(cx).window_decorations {
            settings::WindowDecorations::Server => gpui::WindowDecorations::Server,
            settings::WindowDecorations::Client => gpui::WindowDecorations::Client,
        },
    };

    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(9.0), px(9.0))),
        }),
        window_bounds: None,
        focus: false,
        show: false,
        kind: WindowKind::Normal,
        is_movable: true,
        display_id: display.map(|display| display.id()),
        window_background: cx.theme().window_background_appearance(),
        app_id: Some(app_id.to_owned()),
        window_decorations: Some(window_decorations),
        window_min_size: Some(Size {
            width: px(360.0),
            height: px(240.0),
        }),
        ..Default::default()
    }
}

pub fn initialize_workspace(_app_state: Arc<AppState>, cx: &mut App) {
    let mut _on_close_subscription = bind_on_window_closed(cx);
    cx.observe_global::<SettingsStore>(move |cx| {
        _ = _on_close_subscription.is_some();
        _on_close_subscription = bind_on_window_closed(cx);
    })
    .detach();

    cx.observe_new(move |_multi_workspace: &mut MultiWorkspace, window, cx| {
        let Some(window) = window else {
            return;
        };

        let multi_workspace_handle = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            multi_workspace_handle
                .update(cx, |multi_workspace, cx| {
                    multi_workspace.close_window(&CloseWindow, window, cx);
                    false
                })
                .unwrap_or(true)
        });

        let multi_workspace_handle = cx.entity();
        cx.subscribe_in(
            &multi_workspace_handle,
            window,
            |this, _multi_workspace, event: &workspace::MultiWorkspaceEvent, window, cx| {
                let workspace::MultiWorkspaceEvent::ActiveWorkspaceChanged { source_workspace } =
                    event
                else {
                    return;
                };

                let active_workspace = this.workspace().clone();
                let source_workspace = source_workspace.clone();
                active_workspace.update(cx, |workspace, cx| {
                    if let Some(ref source) = source_workspace {
                        if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                            panel.update(cx, |panel, cx| {
                                panel.initialize_from_source_workspace_if_needed(
                                    source.clone(),
                                    window,
                                    cx,
                                );
                            });
                        }
                    }

                    ensure_agent_panel_for_workspace(workspace, source_workspace, window, cx)
                        .detach_and_log_err(cx);
                });
            },
        )
        .detach();

        let window_handle = window.window_handle();
        cx.defer(move |cx| {
            window_handle
                .update(cx, |_, window, cx| {
                    let sidebar =
                        cx.new(|cx| Sidebar::new(multi_workspace_handle.clone(), window, cx));
                    multi_workspace_handle.update(cx, |multi_workspace, cx| {
                        multi_workspace.register_sidebar(sidebar, cx);
                    });
                })
                .ok();
        });
    })
    .detach();

    cx.observe_new(move |workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };

        let workspace_handle = cx.entity();
        let center_pane = workspace.active_pane().clone();
        initialize_pane(workspace, &center_pane, window, cx);

        cx.subscribe_in(&workspace_handle, window, {
            move |workspace, _, event, window, cx| match event {
                workspace::Event::PaneAdded(pane) => {
                    initialize_pane(workspace, pane, window, cx);
                }
                workspace::Event::OpenBundledFile {
                    text,
                    title,
                    language,
                } => open_bundled_file(workspace, text.clone(), title, language, window, cx),
                _ => {}
            }
        })
        .detach();

        let search_button = cx.new(|_| search::search_status_button::SearchButton::new());
        let active_file_name = cx.new(|_| workspace::active_file_name::ActiveFileName::new());
        let active_buffer_encoding =
            cx.new(|_| encoding_selector::ActiveBufferEncoding::new(workspace));
        let cursor_position =
            cx.new(|_| go_to_line::cursor_position::CursorPosition::new(workspace));
        let line_ending_indicator =
            cx.new(|_| line_ending_selector::LineEndingIndicator::default());
        let merge_conflict_indicator =
            cx.new(|cx| git_ui::MergeConflictIndicator::new(workspace, cx));
        let image_info = cx.new(|_cx| ImageInfo::new(workspace));

        workspace.status_bar().update(cx, |status_bar, cx| {
            status_bar.add_left_item(search_button, window, cx);
            status_bar.add_left_item(active_file_name, window, cx);
            status_bar.add_left_item(merge_conflict_indicator, window, cx);
            status_bar.add_right_item(active_buffer_encoding, window, cx);
            status_bar.add_right_item(line_ending_indicator, window, cx);
            status_bar.add_right_item(cursor_position, window, cx);
            status_bar.add_right_item(image_info, window, cx);
        });

        let panels_task = initialize_panels(window, cx);
        workspace.set_panels_task(panels_task);

        if !workspace.has_active_modal(window, cx) {
            workspace.focus_handle(cx).focus(window, cx);
        }
    })
    .detach();
}

fn bind_on_window_closed(cx: &mut App) -> Option<gpui::Subscription> {
    #[cfg(target_os = "macos")]
    {
        WorkspaceSettings::get_global(cx)
            .on_last_window_closed
            .is_quit_app()
            .then(|| {
                cx.on_window_closed(|cx, _window_id| {
                    if cx.windows().is_empty() {
                        cx.quit();
                    }
                })
            })
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(cx.on_window_closed(|cx, _window_id| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        }))
    }
}

fn initialize_panels(window: &mut Window, cx: &mut Context<Workspace>) -> Task<anyhow::Result<()>> {
    cx.spawn_in(window, async move |workspace_handle, cx| {
        let project_panel = ProjectPanel::load(workspace_handle.clone(), cx.clone());
        let outline_panel = OutlinePanel::load(workspace_handle.clone(), cx.clone());
        let git_panel = GitPanel::load(workspace_handle.clone(), cx.clone());

        async fn add_panel_when_ready(
            panel_task: impl Future<Output = anyhow::Result<Entity<impl Panel>>> + 'static,
            workspace_handle: WeakEntity<Workspace>,
            mut cx: AsyncWindowContext,
        ) {
            if let Some(panel) = panel_task.await.context("failed to load panel").log_err() {
                workspace_handle
                    .update_in(&mut cx, |workspace, window, cx| {
                        workspace.add_panel(panel, window, cx);
                    })
                    .log_err();
            }
        }

        futures::join!(
            add_panel_when_ready(project_panel, workspace_handle.clone(), cx.clone()),
            add_panel_when_ready(outline_panel, workspace_handle.clone(), cx.clone()),
            add_panel_when_ready(git_panel, workspace_handle.clone(), cx.clone()),
            initialize_agent_panel(workspace_handle, cx.clone()).map(|r| r.log_err()),
        );

        anyhow::Ok(())
    })
}

fn setup_or_teardown_ai_panel<P: Panel>(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
    load_panel: impl FnOnce(
        WeakEntity<Workspace>,
        AsyncWindowContext,
    ) -> Task<anyhow::Result<Entity<P>>>
    + 'static,
) -> Task<anyhow::Result<()>> {
    let disable_ai = SettingsStore::global(cx)
        .get::<DisableAiSettings>(None)
        .disable_ai
        || cfg!(test);
    let existing_panel = workspace.panel::<P>(cx);
    match (disable_ai, existing_panel) {
        (false, None) => cx.spawn_in(window, async move |workspace, cx| {
            let panel = load_panel(workspace.clone(), cx.clone()).await?;
            workspace.update_in(cx, |workspace, window, cx| {
                let disable_ai = SettingsStore::global(cx)
                    .get::<DisableAiSettings>(None)
                    .disable_ai;
                let have_panel = workspace.panel::<P>(cx).is_some();
                if !disable_ai && !have_panel {
                    workspace.add_panel(panel, window, cx);
                }
            })
        }),
        (true, Some(existing_panel)) => {
            workspace.remove_panel::<P>(&existing_panel, window, cx);
            Task::ready(Ok(()))
        }
        _ => Task::ready(Ok(())),
    }
}

fn ensure_agent_panel_for_workspace(
    workspace: &mut Workspace,
    source_workspace: Option<WeakEntity<Workspace>>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Task<anyhow::Result<()>> {
    let task = setup_or_teardown_ai_panel(workspace, window, cx, move |workspace, cx| {
        AgentPanel::load(workspace, cx)
    });

    cx.spawn_in(window, async move |workspace, cx| {
        task.await?;
        workspace.update_in(cx, |workspace, window, cx| {
            if let Some(source_workspace) = source_workspace.clone() {
                if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                    panel.update(cx, |panel, cx| {
                        panel.initialize_from_source_workspace_if_needed(
                            source_workspace,
                            window,
                            cx,
                        );
                    });
                }
            }
        })
    })
}

async fn initialize_agent_panel(
    workspace_handle: WeakEntity<Workspace>,
    mut cx: AsyncWindowContext,
) -> anyhow::Result<()> {
    workspace_handle
        .update_in(&mut cx, |workspace, window, cx| {
            ensure_agent_panel_for_workspace(workspace, None, window, cx)
        })?
        .await?;

    workspace_handle.update_in(&mut cx, |workspace, window, cx| {
        cx.observe_global_in::<SettingsStore>(window, move |workspace, window, cx| {
            ensure_agent_panel_for_workspace(workspace, None, window, cx).detach_and_log_err(cx);
        })
        .detach();

        if !cfg!(test) {
            workspace
                .register_action(AgentPanel::toggle_focus)
                .register_action(AgentPanel::focus)
                .register_action(AgentPanel::toggle)
                .register_action(agent_ui::InlineAssistant::inline_assist);
        }
    })?;

    anyhow::Ok(())
}

fn initialize_pane(
    workspace: &Workspace,
    pane: &Entity<Pane>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    pane.update(cx, |pane, cx| {
        pane.toolbar().update(cx, |toolbar, cx| {
            let breadcrumbs = cx.new(|_| breadcrumbs::Breadcrumbs::new());
            toolbar.add_item(breadcrumbs, window, cx);

            let writing_toolbar = cx.new(|_| WritingToolbar::new(workspace));
            toolbar.add_item(writing_toolbar, window, cx);

            let project_search_bar = cx.new(|_| ProjectSearchBar::new());
            toolbar.add_item(project_search_bar, window, cx);
        });
    });
}

pub fn app_menus(_cx: &mut App) -> Vec<Menu> {
    vec![
        Menu {
            name: "Aleph".into(),
            disabled: false,
            items: vec![
                MenuItem::action("About Aleph", About),
                MenuItem::separator(),
                MenuItem::submenu(Menu::new("Settings").items([
                    MenuItem::action("Open Settings File", OpenSettingsFile),
                    MenuItem::action("Open Default Settings", OpenDefaultSettings),
                    MenuItem::separator(),
                    MenuItem::action("Open Keymap File", zed_actions::OpenKeymapFile),
                    MenuItem::action("Open Default Key Bindings", zed_actions::OpenDefaultKeymap),
                    MenuItem::separator(),
                    MenuItem::action(
                        "Select Theme...",
                        zed_actions::theme_selector::Toggle::default(),
                    ),
                    MenuItem::action(
                        "Select Icon Theme...",
                        zed_actions::icon_theme_selector::Toggle::default(),
                    ),
                ])),
                MenuItem::separator(),
                MenuItem::action("Extensions", zed_actions::Extensions::default()),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::action("Hide Aleph", Hide),
                #[cfg(target_os = "macos")]
                MenuItem::action("Hide Others", HideOthers),
                #[cfg(target_os = "macos")]
                MenuItem::action("Show All", ShowAll),
                MenuItem::separator(),
                MenuItem::action("Quit Aleph", Quit),
            ],
        },
        Menu {
            name: "File".into(),
            disabled: false,
            items: vec![
                MenuItem::action("New", NewFile),
                MenuItem::action("New Window", NewWindow),
                MenuItem::separator(),
                MenuItem::action("Open…", workspace::Open::default()),
                MenuItem::action(
                    "Open Recent...",
                    zed_actions::OpenRecent {
                        create_new_window: false,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("Add Folder to Project…", workspace::AddFolderToProject),
                MenuItem::separator(),
                MenuItem::action("Save", workspace::Save { save_intent: None }),
                MenuItem::action("Save As…", workspace::SaveAs),
                MenuItem::action("Save All", workspace::SaveAll { save_intent: None }),
                MenuItem::separator(),
                MenuItem::action(
                    "Close Editor",
                    workspace::CloseActiveItem {
                        save_intent: None,
                        close_pinned: true,
                    },
                ),
                MenuItem::action("Close Window", CloseWindow),
            ],
        },
        Menu {
            name: "Edit".into(),
            disabled: false,
            items: vec![
                MenuItem::os_action("Undo", editor::actions::Undo, OsAction::Undo),
                MenuItem::os_action("Redo", editor::actions::Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action("Cut", editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action("Copy", editor::actions::Copy, OsAction::Copy),
                MenuItem::os_action("Paste", editor::actions::Paste, OsAction::Paste),
                MenuItem::separator(),
                MenuItem::action("Find", search::buffer_search::Deploy::find()),
                MenuItem::action("Find in Project", workspace::DeploySearch::default()),
                MenuItem::separator(),
                MenuItem::action(
                    "Toggle Line Comment",
                    editor::actions::ToggleComments::default(),
                ),
            ],
        },
        Menu {
            name: "Selection".into(),
            disabled: false,
            items: vec![
                MenuItem::os_action(
                    "Select All",
                    editor::actions::SelectAll,
                    OsAction::SelectAll,
                ),
                MenuItem::action("Expand Selection", editor::actions::SelectLargerSyntaxNode),
                MenuItem::action("Shrink Selection", editor::actions::SelectSmallerSyntaxNode),
                MenuItem::separator(),
                MenuItem::action(
                    "Add Cursor Above",
                    editor::actions::AddSelectionAbove {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    "Add Cursor Below",
                    editor::actions::AddSelectionBelow {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    "Select Next Occurrence",
                    editor::actions::SelectNext {
                        replace_newest: false,
                    },
                ),
                MenuItem::action("Select All Occurrences", editor::actions::SelectAllMatches),
                MenuItem::separator(),
                MenuItem::action("Move Line Up", editor::actions::MoveLineUp),
                MenuItem::action("Move Line Down", editor::actions::MoveLineDown),
                MenuItem::action("Duplicate Selection", editor::actions::DuplicateLineDown),
            ],
        },
        Menu {
            name: "View".into(),
            disabled: false,
            items: vec![
                MenuItem::action(
                    "Zoom In",
                    zed_actions::IncreaseBufferFontSize { persist: false },
                ),
                MenuItem::action(
                    "Zoom Out",
                    zed_actions::DecreaseBufferFontSize { persist: false },
                ),
                MenuItem::action(
                    "Reset Zoom",
                    zed_actions::ResetBufferFontSize { persist: false },
                ),
                MenuItem::separator(),
                MenuItem::action("Toggle Left Dock", workspace::ToggleLeftDock),
                MenuItem::action("Toggle Right Dock", workspace::ToggleRightDock),
                MenuItem::action("Toggle Bottom Dock", workspace::ToggleBottomDock),
                MenuItem::separator(),
                MenuItem::action("Project Panel", zed_actions::project_panel::ToggleFocus),
                MenuItem::action("Outline Panel", outline_panel::ToggleFocus),
            ],
        },
        Menu {
            name: "Go".into(),
            disabled: false,
            items: vec![
                MenuItem::action("Back", workspace::GoBack),
                MenuItem::action("Forward", workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action("Command Palette...", zed_actions::command_palette::Toggle),
                MenuItem::separator(),
                MenuItem::action("Go to File...", workspace::ToggleFileFinder::default()),
                MenuItem::action("Go to Heading...", zed_actions::outline::ToggleOutline),
                MenuItem::action("Go to Line/Column...", editor::actions::ToggleGoToLine),
            ],
        },
    ]
}

pub fn watch_settings_files(fs: Arc<dyn Fs>, cx: &mut App) {
    SettingsStore::update_global(cx, move |store, cx| {
        store.watch_settings_files(fs, cx, |_settings_file, result, _cx| {
            if let Some(error) = result.parse_error() {
                log::error!("Settings error: {}", error);
            }
        });
    });
}

pub fn handle_keymap_file_changes(
    mut user_keymap_file_rx: mpsc::UnboundedReceiver<String>,
    user_keymap_watcher: gpui::Task<()>,
    cx: &mut App,
) {
    let (base_keymap_tx, mut base_keymap_rx) = mpsc::unbounded();
    let mut old_base_keymap = *BaseKeymap::get_global(cx);

    cx.observe_global::<SettingsStore>(move |cx| {
        let new_base_keymap = *BaseKeymap::get_global(cx);
        if new_base_keymap != old_base_keymap {
            old_base_keymap = new_base_keymap;
            base_keymap_tx.unbounded_send(()).unwrap();
        }
    })
    .detach();

    load_default_keymap(cx);

    cx.spawn(async move |cx| {
        let _user_keymap_watcher = user_keymap_watcher;
        let mut user_keymap_content = String::new();
        loop {
            select_biased! {
                _ = base_keymap_rx.next() => {},
                content = user_keymap_file_rx.next() => {
                    if let Some(content) = content {
                        user_keymap_content = content;
                    }
                }
            };
            cx.update(|cx| {
                let load_result = KeymapFile::load(&user_keymap_content, cx);
                match load_result {
                    KeymapFileLoadResult::Success { key_bindings } => {
                        reload_keymaps(cx, key_bindings);
                    }
                    KeymapFileLoadResult::SomeFailedToLoad {
                        key_bindings,
                        error_message,
                    } => {
                        if !key_bindings.is_empty() {
                            reload_keymaps(cx, key_bindings);
                        }
                        log::error!("Keymap loading error: {}", error_message);
                    }
                    KeymapFileLoadResult::JsonParseFailure { error } => {
                        log::error!("Keymap JSON parse error: {}", error);
                    }
                }
            });
        }
    })
    .detach();
}

fn load_default_keymap(cx: &mut App) {
    let base_keymap = *BaseKeymap::get_global(cx);
    if base_keymap != BaseKeymap::None {
        if let Ok(bindings) = KeymapFile::load_asset_allow_partial_failure(DEFAULT_KEYMAP_PATH, cx)
        {
            cx.bind_keys(bindings);
        }

        if let Some(asset_path) = base_keymap.asset_path() {
            if let Ok(bindings) = KeymapFile::load_asset_allow_partial_failure(asset_path, cx) {
                cx.bind_keys(bindings);
            }
        }
    }
}

fn reload_keymaps(cx: &mut App, mut user_key_bindings: Vec<KeyBinding>) {
    cx.clear_key_bindings();
    load_default_keymap(cx);

    for key_binding in &mut user_key_bindings {
        key_binding.set_meta(KeybindSource::User.meta());
    }
    cx.bind_keys(user_key_bindings);

    let menus = app_menus(cx);
    cx.set_menus(menus);
}

fn open_settings_file(
    path: &'static PathBuf,
    default_content: impl FnOnce() -> rope::Rope + Send + 'static,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    workspace::create_and_open_local_file(path.as_path(), window, cx, default_content)
        .detach_and_log_err(cx);
}

fn open_bundled_file(
    workspace: &mut Workspace,
    text: std::borrow::Cow<'static, str>,
    title: &'static str,
    language: &'static str,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let existing = workspace
        .items_of_type::<editor::Editor>(cx)
        .find(|editor| {
            editor.read_with(cx, |editor, cx| {
                editor.read_only(cx)
                    && editor.title(cx).as_ref() == title
                    && editor
                        .buffer()
                        .read(cx)
                        .as_singleton()
                        .is_some_and(|buffer| buffer.read(cx).file().is_none())
            })
        });
    if let Some(existing) = existing {
        workspace.activate_item(&existing, true, true, window, cx);
        return;
    }

    let language = workspace.app_state().languages.language_for_name(language);
    cx.spawn_in(window, async move |workspace, cx| {
        let language = language.await.log_err();
        workspace
            .update_in(cx, move |workspace, window, cx| {
                let project = workspace.project().clone();
                let buffer = project.update(cx, move |project, cx| {
                    project.create_buffer(language, false, cx)
                });
                cx.spawn_in(window, async move |workspace, cx| {
                    let buffer = buffer.await?;
                    buffer.update(cx, |buffer, cx| {
                        buffer.set_text(text.into_owned(), cx);
                        buffer.set_capability(Capability::ReadOnly, cx);
                    });
                    let buffer = cx.new(|cx| {
                        multi_buffer::MultiBuffer::singleton(buffer, cx).with_title(title.into())
                    });
                    workspace.update_in(cx, |workspace, window, cx| {
                        workspace.add_item_to_active_pane(
                            Box::new(cx.new(|cx| {
                                let mut editor = editor::Editor::for_multibuffer(
                                    buffer,
                                    Some(project.clone()),
                                    window,
                                    cx,
                                );
                                editor.set_read_only(true);
                                editor.set_breadcrumb_header(title.into());
                                editor
                            })),
                            None,
                            true,
                            window,
                            cx,
                        )
                    })
                })
            })?
            .await
    })
    .detach_and_log_err(cx);
}
