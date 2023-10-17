use crate::logs::FormattedLog;
use crate::logs::ParsableLog;
use std::collections::HashMap;

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlformat::{Indent, QueryParams};
use termcolor::Color;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KnexLog {
    sql: String,
    bindings: Option<Value>,
}
impl ParsableLog for KnexLog {
    fn format_compact(&self) -> FormattedLog {
        let time = Local::now();
        let bindings = self
            .bindings
            .clone()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| {
                if x.is_string() {
                    format!("'{}'", x.as_str().unwrap().to_string())
                } else {
                    x.to_string()
                }
            })
            .collect();
        let qs = QueryParams::Indexed(bindings);
        let msg = sqlformat::format(
            &self.sql.clone(),
            &qs,
            sqlformat::FormatOptions {
                indent: Indent::Spaces(2),
                uppercase: true,
                lines_between_queries: 0,
            },
        );
        FormattedLog {
            date: time,
            msg: format!(
                "SQL query (might be different than the actual query):\n{}",
                msg
            ),
            extra: None,
            color_overwrite: Some(Color::Yellow),
        }
    }

    fn from_line(line: &str) -> Option<Self> {
        match serde_json::from_str::<KnexLog>(&line) {
            Ok(r) => Some(r),
            Err(_e) => {
                // println!("ERRRRRR {}", e);
                None
            }
        }
    }

    fn format_detailed(&self) -> FormattedLog {
        let time = Local::now();
        let msg = self.sql.clone();
        let mut bindings_map: HashMap<String, Value> = HashMap::new();
        if let Some(b) = &self.bindings {
            bindings_map.insert("bidings".to_string(), b.clone());
        }
        FormattedLog {
            date: time,
            msg,
            extra: Some(bindings_map),
            color_overwrite: Some(Color::Blue),
        }
    }
}
