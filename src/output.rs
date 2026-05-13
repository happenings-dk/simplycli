use anyhow::Result;
use serde_json::Value;

pub enum ApiOutput {
    Json(Value),
    Text(String),
}

pub fn print_output(output: ApiOutput, force_json: bool) -> Result<()> {
    match output {
        ApiOutput::Json(value) => println!("{}", serde_json::to_string_pretty(&value)?),
        ApiOutput::Text(text) if force_json => {
            println!("{}", serde_json::to_string_pretty(&Value::String(text))?);
        }
        ApiOutput::Text(text) => println!("{text}"),
    }

    Ok(())
}
