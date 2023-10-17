use crate::logs::FormattedLog;
use crate::logs::ParsableLog;
use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

fn default_none<T>() -> Option<T> {
    None
}

/// Represents the internal data part of a standard log
/// This is the trickiest part of the log. It can contain virtually anything
/// And the data that's buried here is the most valuable for us (errors, who requested,
/// body returned, etc)
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InternalDataLog {
    internal_context: Option<HashMap<String, Value>>,
    context: HashMap<String, Value>,
    status_code: Option<i32>,
    name: Option<String>,
    http_code: Option<i32>,
    err: Option<HashMap<String, Value>>,
    req: Option<HashMap<String, Value>>,
    body: Option<HashMap<String, Value>>,
    error: Option<HashMap<String, Value>>,
    options: Option<HashMap<String, Value>>,
    response: Option<HashMap<String, Value>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InternalLogResponse {
    status_code: i32,
    request: Option<HashMap<String, Value>>,
    body: Option<HashMap<String, Value>>,
    headers: Option<HashMap<String, Value>>,
}

/// A message can be either a string or an object. So we have 2 structs that we attempt to parse
/// the message
#[derive(Serialize, Deserialize, Debug)]
struct StringMsgLog {
    msg: String,
}

/// A message can be either a string or an object. So we have 2 structs that we attempt to parse
/// the message
#[derive(Serialize, Deserialize, Debug)]
struct StringObjLog {
    msg: HashMap<String, Value>,
}

/// Main log used by us. It contains detailed information that is useful at production envs, but
/// that isn't necessarily full of useful information when developing. The way this works it'll
/// "match" to any json object that contains an integer "level" property
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StandardLog {
    level: u8,
    // In ms
    time: Option<u64>,
    pid: Option<i32>,
    hostname: Option<String>,
    name: Option<String>,
    data: Option<InternalDataLog>,

    // Skip both msg and msg_obj as we'll manually try each one and populate accordingly
    #[serde(skip, default = "default_none")]
    msg: Option<String>,
    #[serde(skip, default = "default_none")]
    msg_obj: Option<HashMap<String, Value>>,
}

/// Implements the Parsable log trait for standardlog which ties our representation with the rest
/// of the system and allows standard logs to be properly logged to the user using their
/// configuration
impl ParsableLog for StandardLog {
    fn format_compact(&self) -> FormattedLog {
        // We might have a time, but if something happens and we can't find it, use now
        let time = match self.time {
            Some(t) => DateTime::from_timestamp(t as i64 / 1000, 0)
                .unwrap()
                .with_timezone(&Local::now().timezone()),
            None => Local::now(),
        };

        // Looks for a url in the request body. Useful to see exactly what url was called. So our
        // log message could be `GET /users`, but we also want to log `GET /users?bla=xxx` and the
        // easiest place to get that is in data.req.url
        let url = match &self.data {
            Some(d) => {
                if let Some(req) = &d.req {
                    req.get("url")
                } else {
                    None
                }
            }
            None => None,
        };

        let mut log = "".to_string();
        let mut extra: Option<HashMap<String, Value>> = None;

        if let Some(data) = &self.data {
            if let Some(name) = &data.name {
                log.push_str(&format!(" - Responding with error {}", name));
            }
        }
        if let Some(data) = &self.data {
            if let Some(http_code) = &data.http_code {
                log.push_str(&format!(" - HTTP code {}", http_code));
            }
        }
        if let Some(url) = url {
            log.push_str(&format!(" - {}", url.as_str().unwrap_or("")));
        }

        if let Some(msg) = &self.msg {
            log.push_str(&format!(" - {}", msg));
        }

        if let Some(msg) = &self.msg_obj {
            log.push_str(&serde_json::to_string_pretty(&msg).unwrap_or("".to_string()));
        }

        // Useful for endpoints that respond with some sort of data in the body. This is usually
        // were we log them
        if let Some(data) = &self.data {
            if let Some(body) = &data.body {
                extra = Some(body.clone());
                log.push_str(&" - Responding with");
            }
        }

        FormattedLog {
            date: time,
            msg: log,
            extra,
            color_overwrite: None,
        }
    }

    fn format_detailed(&self) -> FormattedLog {
        let time = match self.time {
            Some(t) => DateTime::from_timestamp(t as i64 / 1000, 0)
                .unwrap()
                .with_timezone(&Local::now().timezone()),
            None => Local::now(),
        };

        // Looks for a url in the request body. Useful to see exactly what url was called. So our
        // log message could be `GET /users`, but we also want to log `GET /users?bla=xxx` and the
        // easiest place to get that is in data.req.url
        let url = match &self.data {
            Some(d) => {
                if let Some(req) = &d.req {
                    req.get("url")
                } else {
                    None
                }
            }
            None => None,
        };

        let mut log = "".to_string();
        let mut extra: HashMap<String, Value> = HashMap::new();

        if let Some(data) = &self.data {
            if let Some(name) = &data.name {
                log.push_str(&format!(" - Responding with error {}", name));
            }
        }
        if let Some(data) = &self.data {
            if let Some(http_code) = &data.http_code {
                log.push_str(&format!(" - HTTP code {}", http_code));
            }
        }
        if let Some(url) = url {
            log.push_str(&format!(" - {}", url.as_str().unwrap_or("")));
        }

        if let Some(msg) = &self.msg {
            log.push_str(&format!(" - {}", msg));
        }

        if let Some(msg) = &self.msg_obj {
            log.push_str(&serde_json::to_string_pretty(&msg).unwrap_or("".to_string()));
        }

        // println!("What's available {:#?}", self);

        // Useful for endpoints that respond with some sort of data in the body. This is usually
        // were we log them
        if let Some(data) = &self.data {
            if let Some(internal_context) = &data.internal_context {
                extra.insert("internalContext".to_string(), Value::Object(Map::new()));
                for (k, v) in internal_context.iter() {
                    extra
                        .get_mut("internalContext")
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .insert(k.clone(), v.clone());
                }
            }
            if let Some(req) = &data.req {
                extra.insert("req".to_string(), Value::Object(Map::new()));
                for (k, v) in req.iter() {
                    extra
                        .get_mut("req")
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .insert(k.clone(), v.clone());
                }
            }
            if let Some(body) = &data.body {
                extra.insert("dataBody".to_string(), Value::Object(Map::new()));
                for (k, v) in body.iter() {
                    extra
                        .get_mut("dataBody")
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .insert(k.clone(), v.clone());
                }
            }
        }

        FormattedLog {
            date: time,
            msg: log,
            extra: if extra.len() > 0 { Some(extra) } else { None },
            color_overwrite: None,
        }
    }

    fn from_line(line: &str) -> Option<Self> {
        match serde_json::from_str::<StandardLog>(&line) {
            Ok(mut standard_log) => {
                // Standard log does not yet contain a message because it could be a string or an object. Let's try
                // to parse that
                if let Ok(str_msg) = serde_json::from_str::<StringMsgLog>(&line) {
                    standard_log.msg = Some(str_msg.msg);
                } else if let Ok(msg_obj) = serde_json::from_str::<StringObjLog>(&line) {
                    standard_log.msg_obj = Some(msg_obj.msg);
                }

                Some(standard_log)
            }
            Err(_e) => {
                // println!("ERRRRRR {}", e);
                None
            }
        }
    }
}
