use { 
    client_api::{external::::{api::relative_url, experiences_types::types::{CompressedExperienceEvent, CreateExperienceRequest}, wrappers::Band}, plugin::{PluginData, PluginEventData, PluginTrait}, result::EventResult, style::{Style, StyledView}}, leptos::{html::Input, spawn_local, view, window, Callback, IntoView, View}
};

pub struct Plugin {
    #[allow(unused)]
    plugin_data: PluginData,
}

impl PluginTrait for Plugin {
    async fn new(data: PluginData) -> Self
        where
            Self: Sized {
            Plugin {
                plugin_data: data
            }
    }

    fn get_component(&self, data: PluginEventData) -> EventResult<Box<dyn FnOnce() -> leptos::View>> {
        let data = data.get_data::<CompressedExperienceEvent>()?;
        Ok(Box::new(move || -> View {
            match data {
                CompressedExperienceEvent::Experience(id) => {
                    view! {
                        <div style="display: flex; flex-direction: column;">
                            <img
                                style="width: 100%"
                                src=relative_url(&format!("/api/experience/{}/cover/big", id))
                                    .unwrap()
                                    .to_string()
                            />
                            <Band on:click=Callback::new(move |_| {
                                window()
                                    .location()
                                    .set_href(
                                        &relative_url(&format!("/experience/{}", id))
                                            .unwrap()
                                            .to_string(),
                                    )
                                    .unwrap();
                            })>Open</Band>
                        </div>
                    }.into_view()
                }
                CompressedExperienceEvent::Create(timing) => {
                    let name_ref = leptos::create_node_ref::<Input>();

                    view! {
                        <style>
                            "
                            .name_input {
                             border: none;
                             width: 100%;
                             box-sizing: border-box;
                             background-color: var(--accentColor1);
                             padding: var(--contentSpacing);
                             color: var(--lightColor);
                            }
                            .name_input::placeholder{
                             color: var(--lightColor);
                            }
                            .name_input:focus{
                             outline: none;
                            }"
                        </style>
                        <StyledView>
                            <input ref=name_ref class="name_input" placeholder="Name" />
                            <Band click=Callback::new(move |_| {
                                let timing = timing.clone();
                                spawn_local(async move {
                                    match client_api::api::api_request(
                                            "/plugin/timeline_plugin_experience/create",
                                            &CreateExperienceRequest {
                                                name: name_ref.get().unwrap().value(),
                                                time: timing.clone(),
                                            },
                                        )
                                        .await
                                    {
                                        Ok(()) => {
                                            let _ = window().location().reload();
                                        }
                                        Err(e) => {
                                            let _ = window()
                                                .alert_with_message(
                                                    &format!("Unable to create Experience: {}", e),
                                                );
                                        }
                                    }
                                })
                            })>Create</Band>
                        </StyledView>
                    }.into_view()
                }
            }
        }))
    }

    fn get_style(&self) -> Style {
        Style::Acc1
    }
}