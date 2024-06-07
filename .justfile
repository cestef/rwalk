default: 
    just -l
# Run the program
@run *ARGS="":
    cargo run --quiet -- {{ARGS}}

test:
    cargo nextest run

# Build the program
build OUTPUT="./target/release":
    cargo build --release -Z unstable-options --quiet --out-dir {{OUTPUT}}

# Run the benchmarks against ffuf and dirsearch
bench URL="http://ffuf.me/cd/basic" FILE="common.txt" THREADS="100":
    hyperfine "rwalk {{URL}} {{FILE}} -t {{THREADS}}" "ffuf -u {{URL}}/FUZZ -w {{FILE}} -t {{THREADS}}" "dirsearch -u {{URL}} -w {{FILE}} -t {{THREADS}}"

gif COMMAND="rwalk http://ffuf.me/cd/recursion/ ~/common.txt -d 3":
    asciinema rec assets/rwalk.cast --overwrite -c "{{COMMAND}}" --cols 85 --rows 21
    agg assets/rwalk.cast assets/rwalk.gif --font-family "MesloLGS NF,Apple Symbols" --theme github-dark