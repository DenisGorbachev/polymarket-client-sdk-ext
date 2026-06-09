# Guidelines for `clap`

## Requirements

- Every enum in the project must derive `ValueEnum` with `#[value(rename_all = "kebab-case")]`
