# ICFPC 2025 CLI Tool

A command-line interface for the ICFPC 2025 Aedificium contest.

## Installation

```bash
cargo build --release
```

## Configuration

The tool requires a team ID which can be provided in two ways:

1. **Environment variable**: `ICFPC_TEAM_ID=<team-id>`
2. **.env file**: Create a `.env` file in the project root with:

```env
ICFPC_TEAM_ID=your-team-id
```

## Debug Usage

The CLI tool provides three main commands for interacting with the contest API:

### Select a Problem

```bash
cargo run --bin aedificium -- select <problem-name>
```

Example:

```bash
cargo run --bin aedificium -- select problem1
```

### Explore with Plans

```bash
cargo run --bin aedificium -- explore <plans>
```

Plans should be comma-separated strings:

```bash
cargo run --bin aedificium -- explore '["0325", "1234"]'
```

### Submit a Guess

```bash
cargo run --bin aedificium -- guess '<map-json>'
```

The map should be a JSON string with the following structure:

```bash
cargo run --bin aedificium -- guess '{"rooms":[1,2,3],"startingRoom":1,"connections":[{"from":{"room":1,"door":0},"to":{"room":2,"door":0}}]}'
```

## Options

- `--help, -h`: Show help information
- `--version, -V`: Show version information

## Release Usage

### Build

```bash
cargo build --release
```

### Usage

```bash
# Select a problem
./target/release/aedificium select spaceship

# Explore multiple plans
./target/release/aedificium explore "1,2,3"

# Submit a map guess
./target/release/aedificium guess '{"rooms":[1,2],"startingRoom":1,"connections":[]}'
```
