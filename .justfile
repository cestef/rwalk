default: 
    just -l
# Run the program
run *ARGS="":
    cargo run --quiet --release -- {{ARGS}}

# Build the program
build OUTPUT="./target/release":
    cargo build --release -Z unstable-options --quiet --out-dir {{OUTPUT}}

