use clap::Parser;
use log::info;
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};

mod schema;
mod error;

use crate::error::AppError;
use crate::schema::generate_schema;

/// A CLI tool to generate JSON Schema from JSON input
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input JSON file
    #[clap(value_parser)]
    input: PathBuf,

    /// Output schema file (default: <input>.schema.json)
    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,

    /// Enable strict mode for more detailed schema
    #[clap(short, long)]
    strict: bool,

    /// Pretty print output with indentation
    #[clap(short, long)]
    pretty: bool,

    /// Add value assertions based on input data
    #[clap(short = 'a', long)]
    assert_values: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Processing input file: {:?}", args.input);

    // Ensure input file exists
    if !args.input.exists() {
        return Err(AppError::FileNotFound(args.input.display().to_string()).into());
    }

    // Read input file
    let json_content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {:?}", args.input))?;

    // Parse JSON
    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| AppError::InvalidJson(e.to_string()))?;

    // Generate schema
    let schema = generate_schema(&json_value, args.strict, args.assert_values)?;

    // Determine output path
    let output_path = match args.output {
        Some(path) => path,
        None => {
            let mut path = args.input.clone();
            let stem = path.file_stem().unwrap_or_default();
            let new_name = format!("{}.schema.json", stem.to_string_lossy());
            path.set_file_name(new_name);
            path
        }
    };

    // Serialize schema to JSON
    let schema_json = if args.pretty {
        serde_json::to_string_pretty(&schema)?
    } else {
        serde_json::to_string(&schema)?
    };

    // Write schema to file
    fs::write(&output_path, schema_json)
        .with_context(|| format!("Failed to write schema to file: {:?}", output_path))?;

    println!("Schema generated successfully: {:?}", output_path);

    Ok(())
}
