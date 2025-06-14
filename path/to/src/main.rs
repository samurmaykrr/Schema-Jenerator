import sys;

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
    let schema = generate_schema(&json_value, args.strict, args.assert_values);

    // Add test cases
    if args.strict {
        println!("Strict mode enabled. Schema generated successfully: {:?}", schema);
    } else {
        println!("Schema generated successfully: {:?}", schema);
    }

    Ok(())
}
