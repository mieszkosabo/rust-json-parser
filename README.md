# rust-json-parser

This is an exercise in writing a JSON parser in Rust.
The correctness is validated by the this RFC 8259 compliant [JSON test suite](https://github.com/nst/JSONTestSuite/tree/master?tab=readme-ov-file).

Currently, the parser is able to parse all the files in the test suite except for `n_structure_100000_opening_arrays.json` and `n_structure_open_array_object.json` due to stack overflow (too deep recursion).

## Baseline, tests and benchmarks

The baseline is `JSON.parse` from JavaScript.

### Prerequisites

To run tests you need to have `bun` installed: https://bun.sh/

To run benchmarks I recommend `hyperfine`.

### Running tests

```bash
bun run-tests.ts <path_to_executable>

# example
bun run-tests.ts ./target/release/rust-json-parser
bun run-tests.ts baseline
```

### Running benchmarks

```bash
hyperfine "./target/release/rust-json-parser benchmarks/input.json"
```
