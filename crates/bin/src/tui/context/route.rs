use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Route {
    #[serde(rename = "home")]
    Home {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
    },
    #[serde(rename = "session")]
    Session {
        session_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
    },
    #[serde(rename = "plugin")]
    Plugin {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
}

impl Default for Route {
    fn default() -> Self {
        Route::Home { prompt: None }
    }
}

pub struct RouteContext {
    current: Arc<Mutex<Route>>,
}

impl RouteContext {
    pub fn new(initial: Option<Route>) -> Self {
        Self {
            current: Arc::new(Mutex::new(initial.unwrap_or_default())),
        }
    }

    pub fn data(&self) -> Route {
        self.current.lock().unwrap().clone()
    }

    pub fn navigate(&self, route: Route) {
        *self.current.lock().unwrap() = route;
    }
}

pub fn parse_initial_route(value: &serde_json::Value) -> Option<Route> {
    let obj = value.as_object()?;
    let route_type = obj.get("type")?.as_str()?;
    match route_type {
        "home" => Some(Route::Home { prompt: None }),
        "session" => {
            let session_id = obj.get("sessionID")?.as_str()?;
            Some(Route::Session {
                session_id: session_id.to_string(),
                prompt: None,
            })
        }
        "plugin" => {
            let id = obj.get("id")?.as_str()?;
            Some(Route::Plugin {
                id: id.to_string(),
                data: None,
            })
        }
        _ => None,
    }
}
