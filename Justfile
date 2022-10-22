alias b := build
alias t := test
alias r := run
alias a := audit
alias w := watch
alias d := develop

build:
    cargo build

test:
    cargo test

run:
    cargo run

audit:
    cargo audit

watch:
    cargo watch -x run

develop:
    cargo watch -B 1 -w src -w Cargo.toml -x check -x test -x run