use std::collections::HashMap;
use crate::parser::{Line, parse_env_file};

pub fn sort(path: &str, method: &str) -> Result<String, std::io::Error> {

    let lines: Vec<Line> = parse_env_file(path).expect("failed to parse file");

    let mut output = String::new();
   
    if method == "alpha" {

        let mut key_list = Vec::<Vec<String>>::new();

        for line in &lines {

            if let Line::KeyValue { key, value, .. } = line {
                key_list.push(vec![key.to_string(), value.to_string()]);
            }

        }

        key_list.sort_by(|a, b| a[0].cmp(&b[0]));
        
        for kv in key_list {
            output.push_str(&format!("{}={}\n", kv[0], kv[1]));
        }

        output.push('\n');
        
    } else {

        let mut grouped_keys: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for line in &lines {

            if let Line::KeyValue { key, value, .. } = line {

                let prefix = if key.split('_').nth(1).is_none() {
                    String::from("MISC")
                } else {
                    key.split('_').next().unwrap_or("").to_string()
                };

                grouped_keys.entry(prefix)
                    .or_insert_with(Vec::new)
                    .push((key.clone(), value.clone()));

            }

        }

        for values in grouped_keys.values_mut() {
            values.sort_by(|a, b| a.0.cmp(&b.0));
        }
        
        for (prefix, entries) in grouped_keys {

            output.push_str(&format!("# {}\n", prefix.to_uppercase()));

            for (key, value) in entries {
                output.push_str(&format!("{}={}\n", key, value));
            }

            output.push('\n');

        }

    }

    Ok(output)

}
