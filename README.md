# Schema Jenerator

A powerful command-line tool that generates JSON Schema from JSON input files with multiple output tiers and advanced features.

## Features

### Core Features
- **Tiered Schema Generation**: Choose from 4 different schema complexity levels
- **Batch Processing**: Process multiple JSON files using glob patterns
- **Schema Validation**: Verify generated schemas against JSON Schema meta-schema
- **CLI Completion**: Shell completion support for bash, zsh, fish, elvish, and PowerShell
- **Pretty Printing**: Human-readable formatted output
- **Flexible Output**: Custom output file paths and automatic naming

### Schema Output Tiers

1. **Basic** (`--tier basic`): Minimal schema with just type information
2. **Standard** (`--tier standard`): Balanced schema with basic constraints and required properties for non-null values
3. **Comprehensive** (`--tier comprehensive`): Detailed schema with strict validation, examples, and enhanced constraints
4. **Expert** (`--tier expert`): Maximum validation with metadata, pattern detection, format hints, and comprehensive constraints

### Advanced Features
- Automatic email and URI format detection in Expert tier
- Pattern recognition for common data types
- Intelligent constraint generation based on input data
- Comprehensive error handling and user feedback

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

### Basic Schema Generation

```bash
# Basic usage with standard tier (default)
schema-jenerator input.json

# Choose a specific tier
schema-jenerator input.json --tier basic
schema-jenerator input.json --tier comprehensive
schema-jenerator input.json --tier expert

# With custom output file
schema-jenerator input.json -o custom_output.json

# Pretty-print the output
schema-jenerator input.json --pretty

# Validate the generated schema
schema-jenerator input.json --validate
```

### Batch Processing

```bash
# Process all JSON files in a directory
schema-jenerator "*.json" --batch

# Process files matching a pattern
schema-jenerator "data/*.json" --batch --tier comprehensive

# Combine with other options
schema-jenerator "input/*.json" --batch --pretty --validate
```

### CLI Completion

Generate shell completion scripts:

```bash
# For bash
schema-jenerator completion bash > /etc/bash_completion.d/schema-jenerator

# For zsh
schema-jenerator completion zsh > ~/.zsh/completions/_schema-jenerator

# For fish
schema-jenerator completion fish > ~/.config/fish/completions/schema-jenerator.fish

# For PowerShell
schema-jenerator completion powershell > schema-jenerator.ps1
```

## Error Handling

The tool provides clear error messages for common issues:

- File not found
- Invalid JSON input
- Permission issues when writing output
- Various I/O errors

## Tier Comparison Examples

Input JSON:
```json
{
  "user": {
    "name": "John Doe",
    "email": "john@example.com",
    "age": 30,
    "active": true
  },
  "posts": [
    {
      "title": "Hello World",
      "content": "This is my first post",
      "tags": ["intro", "first"]
    }
  ]
}
```

### Basic Tier Output
```json
{
  "type": "object",
  "properties": {
    "user": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "email": { "type": "string" },
        "age": { "type": "integer" },
        "active": { "type": "boolean" }
      }
    },
    "posts": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "title": { "type": "string" },
          "content": { "type": "string" },
          "tags": {
            "type": "array",
            "items": { "type": "string" }
          }
        }
      }
    }
  }
}
```

### Expert Tier Output
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "title": "Generated Object Schema",
  "description": "Auto-generated schema from JSON data",
  "properties": {
    "user": {
      "type": "object",
      "title": "Generated Object Schema",
      "description": "Auto-generated schema from JSON data",
      "properties": {
        "name": {
          "type": "string",
          "title": "Generated String Schema",
          "minLength": 0,
          "maxLength": 16,
          "examples": ["John Doe"]
        },
        "email": {
          "type": "string",
          "title": "Generated String Schema",
          "format": "email",
          "minLength": 0,
          "maxLength": 32,
          "examples": ["john@example.com"]
        },
        "age": {
          "type": "integer",
          "title": "Generated Integer Schema",
          "minimum": -970,
          "maximum": 1030,
          "multipleOf": 1,
          "examples": [30]
        },
        "active": {
          "type": "boolean",
          "title": "Generated Boolean Schema",
          "description": "Boolean value from JSON data",
          "examples": [true]
        }
      },
      "required": ["name", "email", "age", "active"],
      "additionalProperties": false,
      "minProperties": 1
    },
    "posts": {
      "type": "array",
      "title": "Generated Array Schema",
      "description": "Auto-generated array schema from JSON data",
      "minItems": 1,
      "maxItems": 2,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "title": "Generated Object Schema",
        "description": "Auto-generated schema from JSON data",
        "properties": {
          "title": {
            "type": "string",
            "title": "Generated String Schema",
            "minLength": 0,
            "maxLength": 22,
            "examples": ["Hello World"]
          },
          "content": {
            "type": "string",
            "title": "Generated String Schema",
            "minLength": 0,
            "maxLength": 44,
            "examples": ["This is my first post"]
          },
          "tags": {
            "type": "array",
            "title": "Generated Array Schema",
            "description": "Auto-generated array schema from JSON data",
            "minItems": 1,
            "maxItems": 4,
            "uniqueItems": true,
            "items": {
              "type": "string",
              "title": "Generated String Schema",
              "minLength": 0,
              "maxLength": 10,
              "examples": ["intro"]
            }
          }
        },
        "required": ["title", "content", "tags"],
        "additionalProperties": false,
        "minProperties": 1
      }
    }
  },
  "required": ["user", "posts"],
  "additionalProperties": false,
  "minProperties": 1
}
```

## License

MIT
