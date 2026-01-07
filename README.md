# Fortunate Primes

Calculate Fortunate numbers F(n) using PARI/GP.

## Definition

F(n) = smallest m > 1 such that primorial(n) + m is prime, where primorial(n) = p₁ × p₂ × ... × pₙ

## Requirements

- PARI/GP: `sudo apt install pari-gp`

## Usage

```bash
cargo build --release
./target/release/fortunate-primes <n>
```

## Examples

```text
$ ./target/release/fortunate-primes 10
F(10) = 61
time: 16ms

$ ./target/release/fortunate-primes 500
F(500) = 5167
time: 23.36s
```

Progress is shown after 2 seconds of computation.

## Architecture

- **pari.rs**: PARI/GP subprocess interface
- **search.rs**: Parallel batch coordinator (8 workers, 10K batch size)
- **progress.rs**: Terminal progress reporting with overwrite

## Testing

```bash
cargo test
```

## Performance

Using PARI/GP backend is significantly faster than pure Rust implementations, especially for larger `n`.

Some sample results:

| n    | F(n)  | Batch size | Time   |
| ---- | ----- | ---------- | ------ |
| 500  | 5167  | 800        | 5.7s   |
| 600  | 16187 | 800        | 19.8s  |
| 700  | 12853 | 1600       | 30s    |
| 1079 | 8929  | 800        | 57.28s |
| 1300 | 13457 | 1600       | 3.2m   |
| 1800 | 16229 | 1600       | 8.30m  |
| 2000 | 51137 | 200        | 27.23m |
| 2500 | 25643 | 200        | 27.35m |
