# Schema Jenerator

A command-line tool that generates JSON Schema from JSON input files, following best practices for JSON Schema generation.

## Features

- Generate JSON Schema from any valid JSON file
- Automatically detect and infer types for all JSON elements
- Strict mode for more detailed schema generation (all properties required, no additional properties)
- Value assertions to include examples from the input data
- Pretty-print output option for human-readable schemas
- Custom output file naming
- Comprehensive error handling for invalid JSON inputs

## Installation

From source:

```bash
# Clone the repository
git clone https://github.com/yourusername/schema-jenerator
cd schema-jenerator

# Build and install
cargo install --path .
```

Using Cargo:

```bash
cargo install schema-jenerator
```

## Usage

```bash
# Basic usage (outputs to input.schema.json)
schema-jenerator input.json

# With custom output file
schema-jenerator input.json -o custom_output.json

# Enable strict mode (all properties required, no additional properties)
schema-jenerator input.json --strict

# Pretty-print the output
schema-jenerator input.json --pretty

# Include value assertions based on input data
schema-jenerator input.json --assert-values

# Combine options
schema-jenerator input.json --strict --pretty --assert-values -o schema.json
```

## Error Handling

The tool provides clear error messages for common issues:

- File not found
- Invalid JSON input
- Permission issues when writing output
- Various I/O errors

## Examples

Input JSON:

```json
{
  "name": "John Doe",
  "age": 30,
  "email": "john@example.com",
  "active": true,
  "scores": [95, 87, 92],
  "address": {
    "street": "123 Main St",
    "city": "Anytown",
    "zipcode": "12345"
  }
}
```

Generated Schema (with `--strict --pretty` options):

```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string"
    },
    "age": {
      "type": "integer"
    },
    "email": {
      "type": "string"
    },
    "active": {
      "type": "boolean"
    },
    "scores": {
      "type": "array",
      "items": {
        "type": "integer"
      }
    },
    "address": {
      "type": "object",
      "properties": {
        "street": {
          "type": "string"
        },
        "city": {
          "type": "string"
        },
        "zipcode": {
          "type": "string"
        }
      },
      "required": ["street", "city", "zipcode"],
      "additionalProperties": false
    }
  },
  "required": ["name", "age", "email", "active", "scores", "address"],
  "additionalProperties": false
}
```

## License

MIT
