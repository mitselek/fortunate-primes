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

```
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
