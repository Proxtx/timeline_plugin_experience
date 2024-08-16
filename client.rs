use { 
    leptos::{view, IntoView, View}, 
    serde::{Deserialize, Serialize},
    crate::plugin_manager::PluginData
};

pub struct Plugin {
    #[allow(unused)]
    plugin_data: PluginData,
}

impl crate::Plugin for Plugin {
    async fn new(data: crate::plugin_manager::PluginData) -> Self
        where
            Self: Sized {
            Plugin {
                plugin_data: data
            }
    }

    fn get_component(&self, data: crate::plugin_manager::PluginEventData) -> crate::plugin_manager::EventResult<Box<dyn FnOnce() -> leptos::View>> {
        Ok(Box::new(move || -> View {
            view! { dev }.into_view()
        }))
    }

    fn get_style(&self) -> crate::plugin_manager::Style {
        crate::plugin_manager::Style::Acc2
    }
}