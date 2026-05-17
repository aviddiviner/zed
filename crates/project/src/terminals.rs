use anyhow::Result;
use collections::HashMap;
use gpui::{App, AppContext as _, Context, Entity, Task, WeakEntity};

use async_channel::bounded;
use futures::{FutureExt as _, future::Shared};
use itertools::Itertools as _;
use language::LanguageName;
use settings::{Settings, SettingsLocation};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use task::{Shell, ShellBuilder, ShellKind, SpawnInTerminal};
use terminal::{
    TaskState, TaskStatus, Terminal, TerminalBuilder, insert_zed_terminal_env,
    terminal_settings::TerminalSettings,
};
use util::{
    maybe, rel_path::RelPath,
    command::new_std_command,
};
use crate::ProjectPath;

use crate::Project;

pub struct Terminals {
    pub(crate) local_handles: Vec<WeakEntity<terminal::Terminal>>,
}

fn get_system_shell() -> String {
    #[cfg(target_os = "windows")]
    return "cmd.exe".to_string();

    #[cfg(not(target_os = "windows"))]
    std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
}

impl Project {
    pub fn active_entry_directory(&self, cx: &App) -> Option<PathBuf> {
        let entry_id = self.active_entry()?;
        let worktree = self.worktree_for_entry(entry_id, cx)?;
        let worktree = worktree.read(cx);
        let entry = worktree.entry_for_id(entry_id)?;

        let absolute_path = worktree.absolutize(entry.path.as_ref());
        if entry.is_dir() {
            Some(absolute_path)
        } else {
            absolute_path.parent().map(|p| p.to_path_buf())
        }
    }

    pub fn active_project_directory(&self, cx: &App) -> Option<Arc<Path>> {
        self.active_entry()
            .and_then(|entry_id| self.worktree_for_entry(entry_id, cx))
            .into_iter()
            .chain(self.worktrees(cx))
            .find_map(|tree| tree.read(cx).root_dir())
    }

    pub fn first_project_directory(&self, cx: &App) -> Option<PathBuf> {
        let worktree = self.worktrees(cx).next()?;
        let worktree = worktree.read(cx);
        if worktree.root_entry()?.is_dir() {
            Some(worktree.abs_path().to_path_buf())
        } else {
            None
        }
    }

    pub fn create_terminal_task(
        &mut self,
        spawn_task: SpawnInTerminal,
        cx: &mut Context<Self>,
    ) -> Task<Result<Entity<Terminal>>> {
        let path: Option<Arc<Path>> = if let Some(cwd) = &spawn_task.cwd {
            let cwd = cwd.to_string_lossy();
            let tilde_substituted = shellexpand::tilde(&cwd);
            Some(Arc::from(Path::new(tilde_substituted.as_ref())))
        } else {
            self.active_project_directory(cx)
        };

        let mut settings_location = None;
        if let Some(path) = path.as_ref()
            && let Some((worktree, _)) = self.find_worktree(path, cx)
        {
            settings_location = Some(SettingsLocation {
                worktree_id: worktree.read(cx).id(),
                path: RelPath::empty(),
            });
        }
        let settings = TerminalSettings::get(settings_location, cx).clone();
        let detect_venv = settings.detect_venv.as_option().is_some();

        let (completion_tx, completion_rx) = bounded(1);

        let local_path = path.clone();
        let task_state = Some(TaskState {
            spawned_task: spawn_task.clone(),
            status: TaskStatus::Running,
            completion_rx,
        });
        let shell = get_system_shell();
        let path_style = self.path_style(cx);
        let shell_kind = ShellKind::new(&shell, path_style.is_windows());

        let env_task =
            self.resolve_directory_environment(&shell, path.clone(), cx);

        let project_path_contexts = self
            .active_entry()
            .and_then(|entry_id| self.path_for_entry(entry_id, cx))
            .into_iter()
            .chain(
                self.visible_worktrees(cx)
                    .map(|wt| wt.read(cx).id())
                    .map(|worktree_id| ProjectPath {
                        worktree_id,
                        path: Arc::from(RelPath::empty()),
                    }),
            );
        let toolchains = project_path_contexts
            .filter(|_| detect_venv)
            .map(|p| self.active_toolchain(p, LanguageName::new_static("Python"), cx))
            .collect::<Vec<_>>();
        let lang_registry = self.languages.clone();
        cx.spawn(async move |project, cx| {
            let mut env = env_task.await.unwrap_or_default();
            env.extend(settings.env);

            let activation_script = maybe!(async {
                for toolchain in toolchains {
                    let Some(toolchain) = toolchain.await else {
                        continue;
                    };
                    let language = lang_registry
                        .language_for_name(&toolchain.language_name.0)
                        .await
                        .ok();
                    let lister = language?.toolchain_lister()?;
                    let future =
                        cx.update(|cx| lister.activation_script(&toolchain, shell_kind, cx));
                    return Some(future.await);
                }
                None
            })
            .await
            .unwrap_or_default();

            let builder = project
                .update(cx, move |_, cx| {
                    let format_to_run = || {
                        if let Some(command) = &spawn_task.command {
                            let command = shell_kind.prepend_command_prefix(command);
                            let command = shell_kind.try_quote_prefix_aware(&command);
                            let args = spawn_task
                                .args
                                .iter()
                                .filter_map(|arg| shell_kind.try_quote(&arg));

                            command.into_iter().chain(args).join(" ")
                        } else {
                            format!("exec {shell} -l")
                        }
                    };

                    let (shell, env) = {
                        env.extend(spawn_task.env);
                        match activation_script.clone() {
                            activation_script if !activation_script.is_empty() => {
                                let separator = shell_kind.sequential_commands_separator();
                                let activation_script =
                                    activation_script.join(&format!("{separator} "));
                                let to_run = format_to_run();

                                let arg = format!("{activation_script}{separator} {to_run}");
                                let args = shell_kind.args_for_shell(true, arg);

                                (
                                    Shell::WithArguments {
                                        program: shell,
                                        args,
                                        title_override: None,
                                    },
                                    env,
                                )
                            }
                            _ => (
                                if let Some(program) = spawn_task.command {
                                    Shell::WithArguments {
                                        program,
                                        args: spawn_task.args,
                                        title_override: None,
                                    }
                                } else {
                                    Shell::System
                                },
                                env,
                            ),
                        }
                    };
                    anyhow::Ok(TerminalBuilder::new(
                        local_path.map(|path| path.to_path_buf()),
                        task_state,
                        shell,
                        env,
                        settings.cursor_shape,
                        settings.alternate_scroll,
                        settings.max_scroll_history_lines,
                        settings.path_hyperlink_regexes,
                        settings.path_hyperlink_timeout_ms,
                        false,
                        cx.entity_id().as_u64(),
                        Some(completion_tx),
                        cx,
                        activation_script,
                        path_style,
                    ))
                })??
                .await?;
            project.update(cx, move |this, cx| {
                let terminal_handle = cx.new(|cx| builder.subscribe(cx));

                this.terminals
                    .local_handles
                    .push(terminal_handle.downgrade());

                let id = terminal_handle.entity_id();
                cx.observe_release(&terminal_handle, move |project, _terminal, cx| {
                    let handles = &mut project.terminals.local_handles;

                    if let Some(index) = handles
                        .iter()
                        .position(|terminal| terminal.entity_id() == id)
                    {
                        handles.remove(index);
                        cx.notify();
                    }
                })
                .detach();

                terminal_handle
            })
        })
    }

    pub fn create_terminal_shell(
        &mut self,
        cwd: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> Task<Result<Entity<Terminal>>> {
        self.create_terminal_shell_internal(cwd, cx)
    }

    /// Creates a local terminal in the project directory.
    pub fn create_local_terminal(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Task<Result<Entity<Terminal>>> {
        let working_directory = self.active_project_directory(cx).map(|p| p.to_path_buf());
        self.create_terminal_shell_internal(working_directory, cx)
    }

    fn create_terminal_shell_internal(
        &mut self,
        cwd: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> Task<Result<Entity<Terminal>>> {
        let path = cwd.map(|p| Arc::from(&*p));

        let mut settings_location = None;
        if let Some(path) = path.as_ref()
            && let Some((worktree, _)) = self.find_worktree(path, cx)
        {
            settings_location = Some(SettingsLocation {
                worktree_id: worktree.read(cx).id(),
                path: RelPath::empty(),
            });
        }
        let settings = TerminalSettings::get(settings_location, cx).clone();
        let detect_venv = settings.detect_venv.as_option().is_some();
        let local_path = path.clone();

        let project_path_contexts = self
            .active_entry()
            .and_then(|entry_id| self.path_for_entry(entry_id, cx))
            .into_iter()
            .chain(
                self.visible_worktrees(cx)
                    .map(|wt| wt.read(cx).id())
                    .map(|worktree_id| ProjectPath {
                        worktree_id,
                        path: RelPath::empty().into(),
                    }),
            );
        let toolchains = project_path_contexts
            .filter(|_| detect_venv)
            .map(|p| self.active_toolchain(p, LanguageName::new_static("Python"), cx))
            .collect::<Vec<_>>();
        let shell = settings.shell.program();
        let env_shell = get_system_shell();

        let path_style = self.path_style(cx);

        let env_task =
            self.resolve_directory_environment(&env_shell, path.clone(), cx);

        let lang_registry = self.languages.clone();
        cx.spawn(async move |project, cx| {
            let shell_kind = ShellKind::new(&shell, path_style.is_windows());
            let mut env = env_task.await.unwrap_or_default();
            env.extend(settings.env);

            let activation_script = maybe!(async {
                for toolchain in toolchains {
                    let Some(toolchain) = toolchain.await else {
                        continue;
                    };
                    let language = lang_registry
                        .language_for_name(&toolchain.language_name.0)
                        .await
                        .ok();
                    let lister = language?.toolchain_lister()?;
                    let future =
                        cx.update(|cx| lister.activation_script(&toolchain, shell_kind, cx));
                    return Some(future.await);
                }
                None
            })
            .await
            .unwrap_or_default();

            let builder = project
                .update(cx, move |_, cx| {
                    let (shell, env) = (settings.shell, env);
                    anyhow::Ok(TerminalBuilder::new(
                        local_path.map(|path| path.to_path_buf()),
                        None,
                        shell,
                        env,
                        settings.cursor_shape,
                        settings.alternate_scroll,
                        settings.max_scroll_history_lines,
                        settings.path_hyperlink_regexes,
                        settings.path_hyperlink_timeout_ms,
                        false,
                        cx.entity_id().as_u64(),
                        None,
                        cx,
                        activation_script,
                        path_style,
                    ))
                })??
                .await?;
            project.update(cx, move |this, cx| {
                let terminal_handle = cx.new(|cx| builder.subscribe(cx));

                this.terminals
                    .local_handles
                    .push(terminal_handle.downgrade());

                let id = terminal_handle.entity_id();
                cx.observe_release(&terminal_handle, move |project, _terminal, cx| {
                    let handles = &mut project.terminals.local_handles;

                    if let Some(index) = handles
                        .iter()
                        .position(|terminal| terminal.entity_id() == id)
                    {
                        handles.remove(index);
                        cx.notify();
                    }
                })
                .detach();

                terminal_handle
            })
        })
    }

    pub fn clone_terminal(
        &mut self,
        terminal: &Entity<Terminal>,
        cx: &mut Context<'_, Project>,
        cwd: Option<PathBuf>,
    ) -> Task<Result<Entity<Terminal>>> {
        // We cannot clone the task's terminal, as it will effectively re-spawn the task, which might not be desirable.
        // For now, create a new shell instead.
        if terminal.read(cx).task().is_some() {
            return self.create_terminal_shell(cwd, cx);
        }
        let local_path = cwd;

        let builder = terminal.read(cx).clone_builder(cx, local_path);
        cx.spawn(async |project, cx| {
            let terminal = builder.await?;
            project.update(cx, |project, cx| {
                let terminal_handle = cx.new(|cx| terminal.subscribe(cx));

                project
                    .terminals
                    .local_handles
                    .push(terminal_handle.downgrade());

                let id = terminal_handle.entity_id();
                cx.observe_release(&terminal_handle, move |project, _terminal, cx| {
                    let handles = &mut project.terminals.local_handles;

                    if let Some(index) = handles
                        .iter()
                        .position(|terminal| terminal.entity_id() == id)
                    {
                        handles.remove(index);
                        cx.notify();
                    }
                })
                .detach();

                terminal_handle
            })
        })
    }

    pub fn terminal_settings<'a>(
        &'a self,
        path: &'a Option<PathBuf>,
        cx: &'a App,
    ) -> &'a TerminalSettings {
        let mut settings_location = None;
        if let Some(path) = path.as_ref()
            && let Some((worktree, _)) = self.find_worktree(path, cx)
        {
            settings_location = Some(SettingsLocation {
                worktree_id: worktree.read(cx).id(),
                path: RelPath::empty(),
            });
        }
        TerminalSettings::get(settings_location, cx)
    }

    pub fn exec_in_shell(
        &self,
        command: String,
        cx: &mut Context<Self>,
    ) -> Task<Result<smol::process::Command>> {
        let path = self.first_project_directory(cx);
        let settings = self.terminal_settings(&path, cx).clone();
        let shell = Shell::System;
        let is_windows = self.path_style(cx).is_windows();
        let builder = ShellBuilder::new(&shell, is_windows).non_interactive();
        let (command, args) = builder.build(Some(command), &Vec::new());

        let env_task = self.resolve_directory_environment(
            &shell.program(),
            path.as_ref().map(|p| Arc::from(&**p)),
            cx,
        );

        cx.spawn(async move |project, cx| {
            let mut env = env_task.await.unwrap_or_default();
            env.extend(settings.env);

            project.update(cx, move |_, cx| {
                insert_zed_terminal_env(&mut env, &release_channel::AppVersion::global(cx));
                let mut command = new_std_command(command);
                command.args(args);
                command.envs(env);
                if let Some(path) = path {
                    command.current_dir(path);
                }
                Ok(command)
                .map(|mut process| {
                    util::set_pre_exec_to_start_new_session(&mut process);
                    smol::process::Command::from(process)
                })
            })?
        })
    }

    pub fn local_terminal_handles(&self) -> &Vec<WeakEntity<terminal::Terminal>> {
        &self.terminals.local_handles
    }

    fn resolve_directory_environment(
        &self,
        shell: &str,
        path: Option<Arc<Path>>,
        cx: &mut App,
    ) -> Shared<Task<Option<HashMap<String, String>>>> {
        if let Some(path) = &path {
            let shell = Shell::Program(shell.to_string());
            self.environment
                .update(cx, |project_env, cx| {
                    project_env.local_directory_environment(&shell, path.clone(), cx)
                })
        } else {
            Task::ready(None).shared()
        }
    }
}
