use std::{collections::HashMap, sync::OnceLock};
use std::env;

use colored::{ColoredString, Colorize};

use crate::utils::StrUtil;

fn get_cached_map() -> &'static HashMap<String, Vec<String>> {
    static CACHE: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

    CACHE.get_or_init(|| {
        let mut map = HashMap::new();

        if let Ok(col_str) = env::var("LS_COLORS") {
            let entries: Vec<&str> = col_str.split(":").collect();

            for entry in entries {
                let pair: Vec<&str> = entry.split("=").collect();
                if pair.len() != 2 {
                    continue;
                }

                let name = pair.get(0).unwrap().to_string();
                let color = pair.get(1).unwrap();

                if let Some(index) = name.rfind('.').map(|i| i+1) {
                    let name = name.substr_after(index);

                    let color: Vec<String> = color
                        .split(";")
                        .map(|l| l.to_string())
                        .collect();

                    map.insert(name, color);
                }
            }
        }

        map
    })
}

pub fn compute_color_for(on: ColoredString, text: &String) -> ColoredString {
    match get_cached_map().get(text) {
        None => on,
        Some(codes) => {
            let mut colored_str = on.clone();
            let mut bold = false;

            for code in codes {
                colored_str = compute_on(colored_str, code.as_str());
                bold = true;
            }

            if bold {
                colored_str.bold()
            } else {
                colored_str
            }
        }
    }
}

pub fn compute_on<T: Colorize + Clone>(on: T, code: &str) -> ColoredString where ColoredString: From<T> {
    match code {
        "30" => on.black(),
        "31" => on.red(),
        "32" => on.green(),
        "33" => on.yellow(),
        "34" => on.blue(),
        "35" => on.magenta(),
        "36" => on.cyan(),
        "37" => on.white(),
        "40" => on.on_black(),
        "41" => on.on_red(),
        "42" => on.on_green(),
        "43" => on.on_yellow(),
        "44" => on.on_blue(),
        "45" => on.on_magenta(),
        "46" => on.on_cyan(),
        "47" => on.on_white(),
        "1" => on.bold(),
        "4" => on.underline(),
        "5" => on.blink(),
        "7" => on.reversed(),
        _ => ColoredString::from(on)
    }
}


