use {
    crate::PluginData,
    chrono::{DateTime, TimeDelta},
    serde::Deserialize,
    serde_json::json,
    types::{api::CompressedEvent, timing::TimeRange},
    url::Url
};

#[derive(Deserialize)]
struct ConfigData{
    pub password: String,
    pub url: Url
}

pub struct Plugin {
    _plugin_data: PluginData,
    config: ConfigData
}

impl crate::Plugin for Plugin {
    async fn new(data: PluginData) -> Self
    where
        Self: Sized,
    {
        let config: ConfigData = toml::Value::try_into(
            data.config
                .clone().expect("Failed to init experiences plugin! No config was provided!")
        )
        .unwrap_or_else(|e| panic!("Unable to init experiences plugin! Provided config does not fit the requirements: {}", e));

        Plugin { _plugin_data: data, config }
    }

    fn get_type() -> types::api::AvailablePlugins
    where
        Self: Sized,
    {
        types::api::AvailablePlugins::timeline_plugin_experience
    }

    fn get_compressed_events (&self, query_range: &types::timing::TimeRange) -> std::pin::Pin<Box<dyn futures::Future<Output = types::api::APIResult<Vec<types::api::CompressedEvent>>> + Send>> {
        Box::pin(async move {
            Ok(Vec::new())
        })
    }
}
