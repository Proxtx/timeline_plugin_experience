use { 
    leptos::{view, IntoView, View, Callback, window}, 
    serde::{Deserialize, Serialize},
    crate::plugin_manager::PluginData,
    experiences_navigator_lib::{api::relative_url, wrappers::Band}
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
        let data = data.get_data::<String>()?;
        Ok(Box::new(move || -> View {
            view! {
                <div style="display: flex; flex-direction: column;">
                    <img
                        style="width: 100%"
                        src=relative_url(&format!("/api/experience/{}/cover/big", data))
                            .unwrap()
                            .to_string()
                    />
                    <Band on:click=Callback::new(move |_| {
                        window()
                            .location()
                            .set_href(
                                &relative_url(&format!("/experience/{}", data)).unwrap().to_string(),
                            )
                            .unwrap();
                    })>Open</Band>
                </div>
            }.into_view()
        }))
    }

    fn get_style(&self) -> crate::plugin_manager::Style {
        crate::plugin_manager::Style::Acc1
    }
}