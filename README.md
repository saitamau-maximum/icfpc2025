# ICFPC 2025

The [ICFPC 2025](https://icfpcontest2025.github.io/) solution for team "Maximum".

Team members:

- [@sorachi](https://github.com/sorachi)
- [@thirofoo](https://github.com/thirofoo)
- [@a01sa01to](https://github.com/a01sa01to)

## CLI

### Setup

```bash
cp .env.example .env
```

Fill in the `ICFPC_TEAM_ID` in the `.env` file.

### Build

```bash
cargo build --release
```

### Usage

```bash
# Select a problem
./target/release/aedificium select probatio

# Select a problem from stdin
./target/release/aedificium select < problem.txt

# Explore multiple plans
./target/release/aedificium explore '["0325", "1234"]'

# Explore multiple plans from stdin
./target/release/aedificium explore < plans.json

# Submit a map guess
./target/release/aedificium guess '{"rooms":[1,2],"startingRoom":1,"connections":[]}'

# Submit a map guess from stdin
./target/release/aedificium guess < map.json
```

### Run Solver (eg. Greedy)

```bash
cargo run --bin greedy # debug mode run

cargo build --release # release mode run
./target/release/greedy
```
