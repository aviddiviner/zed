use anyhow::Result;
use gpui::SharedString;
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(RustEmbed)]
#[folder = "src/templates"]
#[include = "*.hbs"]
struct Assets;

pub struct Templates {
    handlebars: Handlebars<'static>,
    system_prompt_override: Option<PathBuf>,
}

impl Templates {
    pub fn new() -> Arc<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_helper("contains", Box::new(contains));
        handlebars.register_embed_templates::<Assets>().unwrap();
        let system_prompt_override = paths::config_dir().join("system_prompt.hbs");
        let system_prompt_override = if system_prompt_override.exists() {
            log::info!(
                "System prompt override found at: {}",
                system_prompt_override.display()
            );
            Some(system_prompt_override)
        } else {
            None
        };
        Arc::new(Self {
            handlebars,
            system_prompt_override,
        })
    }

    /// Render a template by name using the embedded (compiled-in) templates.
    pub fn render_embedded<T: Serialize>(&self, template_name: &str, data: &T) -> Result<String> {
        Ok(self.handlebars.render(template_name, data)?)
    }

    /// Render the system prompt, hot-reloading from the config override if it exists.
    /// Falls back to the embedded template if the override file is absent or unreadable.
    pub fn render_system_prompt<T: Serialize>(&self, data: &T) -> Result<String> {
        if let Some(ref path) = self.system_prompt_override {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    return self
                        .handlebars
                        .render_template(&content, data)
                        .map_err(|err| {
                            anyhow::anyhow!(
                                "Failed to render system prompt override at {}: {}",
                                path.display(),
                                err
                            )
                        });
                }
                Err(err) => {
                    log::warn!(
                        "Could not read system prompt override at {}: {}. Using embedded default.",
                        path.display(),
                        err
                    );
                }
            }
        }
        Ok(self.handlebars.render("system_prompt.hbs", data)?)
    }
}

pub trait Template: Sized {
    const TEMPLATE_NAME: &'static str;

    fn render(&self, templates: &Templates) -> Result<String>
    where
        Self: Serialize + Sized,
    {
        templates.render_embedded(Self::TEMPLATE_NAME, self)
    }
}

#[derive(Serialize)]
pub struct SystemPromptTemplate<'a> {
    #[serde(flatten)]
    pub project: &'a prompt_store::ProjectContext,
    pub available_tools: Vec<SharedString>,
    pub model_name: Option<String>,
    pub current_time: String,
}

impl Template for SystemPromptTemplate<'_> {
    const TEMPLATE_NAME: &'static str = "system_prompt.hbs";

    fn render(&self, templates: &Templates) -> Result<String> {
        templates.render_system_prompt(self)
    }
}

/// Handlebars helper for checking if an item is in a list
fn contains(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let list = h
        .param(0)
        .and_then(|v| v.value().as_array())
        .ok_or_else(|| {
            handlebars::RenderError::new("contains: missing or invalid list parameter")
        })?;
    let query = h.param(1).map(|v| v.value()).ok_or_else(|| {
        handlebars::RenderError::new("contains: missing or invalid query parameter")
    })?;

    if list.contains(query) {
        out.write("true")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_template() {
        let project = prompt_store::ProjectContext::default();
        let template = SystemPromptTemplate {
            project: &project,
            available_tools: vec!["echo".into(), "update_plan".into(), "spawn_agent".into()],
            model_name: Some("test-model".to_string()),
            current_time: "2026-04-29 21:00 CEST".to_string(),
        };
        let templates = Templates::new();
        let rendered = template.render(&templates).unwrap();
        eprintln!("--- RENDERED SYSTEM PROMPT ---\n{}\n--- END ---", rendered);
        assert!(rendered.contains("writing assistant"));
        assert!(rendered.contains("## Tools"));
        assert!(rendered.contains("test-model"));
        assert!(rendered.contains("2026-04-29 21:00 CEST"));
    }
}
