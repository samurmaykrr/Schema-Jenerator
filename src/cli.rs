use clap::{Parser, CommandFactory};
use clap_complete::{generate, Shell};
use log::info;
use std::path::PathBuf;
use std::fs;
use std::io;
use anyhow::{Context, Result};
use glob::glob;

use crate::error::AppError;
use crate::schema::{generate_schema, SchemaOutputTier};
use crate::validation::validate_schema;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Commands>,

    #[clap(value_parser)]
    pub input: Option<PathBuf>,

    #[clap(short, long, value_parser)]
    pub output: Option<PathBuf>,

    #[clap(short = 't', long, value_enum, default_value = "standard")]
    pub tier: SchemaOutputTier,

    #[clap(short, long)]
    pub pretty: bool,

    #[clap(short = 'v', long)]
    pub validate: bool,

    #[clap(short = 'b', long)]
    pub batch: bool,

    #[clap(short = 'c', long)]
    pub config: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub enum Commands {
    Completion {
        #[clap(value_enum)]
        shell: Shell,
    },
}

pub fn run() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if let Some(command) = args.command {
        return handle_command(command);
    }

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

    if !input.exists() {
        return Err(AppError::FileNotFound(input.display().to_string()).into());
    }

    let json_content = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {:?}", input))?;

    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| AppError::InvalidJson(e.to_string()))?;

    let schema = generate_schema(&json_value, &args.tier)?;

    if args.validate {
        validate_schema(&schema)?;
    }

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

    let schema_json = if args.pretty {
        serde_json::to_string_pretty(&schema)?
    } else {
        serde_json::to_string(&schema)?
    };

    fs::write(&output_path, schema_json)
        .with_context(|| format!("Failed to write schema to file: {:?}", output_path))?;

    println!("Schema generated successfully: {:?}", output_path);

    Ok(())
}