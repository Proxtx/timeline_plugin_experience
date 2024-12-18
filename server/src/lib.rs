use {
    server_api::{web::auth, db::{Event, Database}, plugin::PluginData},
    server_api::external::types::external::chrono::TimeDelta,
    server_api::external::futures::{self, StreamExt},
    server_api::external::rocket::{
        http::Status,
        post,
        response::status::{self, Custom},
        routes, Build, Rocket,
    },
    serde::Deserialize,
    server_api::external::types::{
        api::{APIError, APIResult, CompressedEvent},
        timing::Timing,
        available_plugins::AvailablePlugins
    },
    server_api::external::url::Url,
};

use server_api::external::types::external::reqwest;
use serde::Serialize;

use server_api::config::Config;
use server_api::external::rocket::http::CookieJar;
use server_api::external::rocket::serde::json::Json;
use server_api::external::rocket::State;
use server_api::external::types::external::serde_json;
use server_api::plugin::PluginTrait;
use std::sync::Arc;
use server_api::external::toml;
use server_api::external::types;

#[derive(Serialize, Deserialize)]
pub enum CompressedExperienceEvent {
    Experience(String),
    Create(Timing),
}

#[derive(Serialize, Deserialize)]
pub struct CreateExperienceRequest {
    pub name: String,
    pub time: Timing,
}


#[derive(Deserialize, Clone)]
struct ConfigData {
    pub password: String,
    pub url: Url,
}

pub struct Plugin {
    plugin_data: PluginData,
    config: ConfigData,
}

#[derive(Serialize, Deserialize)]
struct DatabaseExperience {
    name: String,
    id: String,
}

impl PluginTrait for Plugin {
    async fn new(data: PluginData) -> Self
    where
        Self: Sized,
    {
        let config: ConfigData = toml::Value::try_into(
            data.config
                .clone().expect("Failed to init experiences plugin! No config was provided!")
        )
        .unwrap_or_else(|e| panic!("Unable to init experiences plugin! Provided config does not fit the requirements: {}", e));

        Plugin {
            plugin_data: data,
            config,
        }
    }

    fn get_type() -> AvailablePlugins
    where
        Self: Sized,
    {
        AvailablePlugins::timeline_plugin_experience
    }

    fn get_routes() -> Vec<server_api::external::rocket::Route>
    where
        Self: Sized,
    {
        routes![create_experience]
    }

    fn rocket_build_access(&self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket.manage(self.config.clone())
    }

    fn get_compressed_events(
        &self,
        query_range: &types::timing::TimeRange,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<Output = types::api::APIResult<Vec<types::api::CompressedEvent>>>
                + Send,
        >,
    > {
        let database = self.plugin_data.database.clone();
        let query_range = query_range.clone();
        let filter = Database::generate_range_filter(&query_range);
        let plg_filter =
            Database::generate_find_plugin_filter(AvailablePlugins::timeline_plugin_experience);
        let filter = Database::combine_documents(filter, plg_filter);
        Box::pin(async move {
            let mut cursor = database
                .get_events::<DatabaseExperience>()
                .find(filter, None)
                .await?;
            let mut result = Vec::new();
            while let Some(v) = cursor.next().await {
                let t = v?;
                result.push(CompressedEvent {
                    title: t.event.name,
                    time: t.timing,
                    data: serde_json::to_value(CompressedExperienceEvent::Experience(t.event.id)).unwrap(),
                })
            }

            let mut current = query_range.start;

            while current < query_range.end {
                let new_current = current
                    .checked_add_signed(TimeDelta::try_hours(1).unwrap())
                    .unwrap();
                let timing = Timing::Instant(current);
                result.push(CompressedEvent {
                    title: "Create Experience".to_string(),
                    time: timing.clone(),
                    data: serde_json::to_value(CompressedExperienceEvent::Create(timing)).unwrap(),
                });
                current = new_current;
            }

            Ok(result)
        })
    }
}

#[post("/create", data = "<request>")]
async fn create_experience(
    request: Json<CreateExperienceRequest>,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
    database: &State<Arc<Database>>,
    experience_config: &State<ConfigData>,
) -> Custom<Json<APIResult<()>>> {
    if auth(cookies, config).is_err() {
        return status::Custom(
            Status::Unauthorized,
            Json(Err(APIError::AuthenticationError)),
        );
    }

    let client = reqwest::Client::new();
    let id = match client.post(experience_config.url.join("/api/experience/create").unwrap()).header(reqwest::header::COOKIE, format!("pwd={}", experience_config.password)).body(serde_json::to_string(&request.0).unwrap()).send().await {
        Ok(v) => {
            match v.text().await {
                Ok(v) => {
                    match serde_json::from_str::<APIResult<String>>(&v) {
                        Ok(v) => match v {
                            Ok(v) => v,
                            Err(e) => {
                                return status::Custom(Status::InternalServerError, Json(Err(APIError::SerdeJsonError(format!("Unable to create experience on experiences server: The experiences server reported an error: {}", e)))))
                            }
                        },
                        Err(e) => {
                            return status::Custom(Status::InternalServerError, Json(Err(APIError::SerdeJsonError(format!("Unable to read response from experiences server: {}", e)))));
                        }
                    }
                },
                Err(e) => {
                    return status::Custom(Status::InternalServerError, Json(Err(APIError::Custom(format!("Unable to read response from experiences server: {}", e)))));
                }
            }
        }
        Err(e) => {
            return status::Custom(Status::InternalServerError, Json(Err(APIError::Custom(format!("Unable to send request to experiences server: {}", e)))));
        }
    };

    match database
        .register_single_event(&Event {
            timing: request.time.clone(),
            id: id.clone(),
            plugin: <Plugin as PluginTrait>::get_type(),
            event: DatabaseExperience {
                name: request.name.clone(),
                id,
            },
        })
        .await
    {
        Ok(_) => status::Custom(Status::Ok, Json(Ok(()))),
        Err(e) => {
            server_api::error::error(
                database.inner().clone(),
                &e,
                Some(<Plugin as PluginTrait>::get_type()),
                &config.error_report_url,
            );
            status::Custom(Status::InternalServerError, Json(Err(e.into())))
        }
    }
}
