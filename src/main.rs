use clap::{Parser, CommandFactory};
use clap_complete::{generate, Shell};
use log::info;
use std::path::PathBuf;
use std::fs;
use std::io;
use anyhow::{Context, Result};
use glob::glob;

mod schema;
mod error;

use crate::error::AppError;
use crate::schema::{generate_schema, SchemaOutputTier};

/// A CLI tool to generate JSON Schema from JSON input
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// Input JSON file or glob pattern
    #[clap(value_parser)]
    input: Option<PathBuf>,

    /// Output schema file (default: <input>.schema.json)
    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,

    /// Schema output tier
    #[clap(short = 't', long, value_enum, default_value = "standard")]
    tier: SchemaOutputTier,

    /// Pretty print output with indentation
    #[clap(short, long)]
    pretty: bool,

    /// Validate generated schema
    #[clap(short = 'v', long)]
    validate: bool,

    /// Process multiple files matching pattern
    #[clap(short = 'b', long)]
    batch: bool,

    /// Configuration file path
    #[clap(short = 'c', long)]
    config: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Generate shell completion scripts
    Completion {
        #[clap(value_enum)]
        shell: Shell,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        return handle_command(command);
    }

    // Require input for schema generation
    let input = args.input.as_ref().ok_or_else(|| {
        AppError::SchemaGeneration("Input file is required for schema generation".to_string())
    })?;

    if args.batch {
        process_batch(input, &args)?;
    } else {
        process_single_file(input, &args)?;
    }

    Ok(())
}

fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Completion { shell } => {
            let mut app = Args::command();
            let app_name = app.get_name().to_string();
            generate(shell, &mut app, app_name, &mut io::stdout());
            Ok(())
        }
    }
}

fn process_batch(input_pattern: &PathBuf, args: &Args) -> Result<()> {
    let pattern = input_pattern.to_string_lossy();
    let mut processed = 0;
    let mut errors = Vec::new();

    for entry in glob(&pattern)
        .map_err(|e| AppError::SchemaGeneration(format!("Invalid glob pattern: {}", e)))?
    {
        match entry {
            Ok(path) => {
                info!("Processing file: {:?}", path);
                match process_single_file(&path, args) {
                    Ok(_) => processed += 1,
                    Err(e) => errors.push(format!("{:?}: {}", path, e)),
                }
            }
            Err(e) => errors.push(format!("Glob error: {}", e)),
        }
    }

    println!("Processed {} files successfully", processed);
    if !errors.is_empty() {
        println!("Errors encountered:");
        for error in errors {
            println!("  {}", error);
        }
    }

    Ok(())
}

fn process_single_file(input: &PathBuf, args: &Args) -> Result<()> {
    info!("Processing input file: {:?}", input);

    // Ensure input file exists
    if !input.exists() {
        return Err(AppError::FileNotFound(input.display().to_string()).into());
    }

    // Read input file
    let json_content = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {:?}", input))?;

    // Parse JSON
    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| AppError::InvalidJson(e.to_string()))?;

    // Generate schema
    let schema = generate_schema(&json_value, &args.tier)?;

    // Validate schema if requested
    if args.validate {
        validate_schema(&schema)?;
    }

    // Determine output path
    let output_path = match &args.output {
        Some(path) => path.clone(),
        None => {
            let mut path = input.clone();
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

fn validate_schema(schema: &serde_json::Value) -> Result<()> {
    use jsonschema::JSONSchema;
    
    // Validate against JSON Schema meta-schema
    let meta_schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema"
    });
    
    match JSONSchema::compile(&meta_schema) {
        Ok(compiled) => {
            if let Err(errors) = compiled.validate(schema) {
                let error_messages: Vec<String> = errors
                    .map(|e| e.to_string())
                    .collect();
                return Err(AppError::SchemaGeneration(
                    format!("Schema validation failed: {}", error_messages.join(", "))
                ).into());
            }
            println!("Schema validation passed");
        }
        Err(e) => {
            return Err(AppError::SchemaGeneration(
                format!("Failed to compile meta-schema: {}", e)
            ).into());
        }
    }
    
    Ok(())
}
