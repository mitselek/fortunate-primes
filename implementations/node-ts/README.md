# Node.js + TypeScript Implementation

**Status**: 🚧 Prototype

## Overview

Node.js implementation with TypeScript using native BigInt for big integer arithmetic and external libraries for primality testing.

## Motivation

JavaScript/TypeScript could offer:

- Accessibility (most popular language by developer count)
- Strong typing with TypeScript
- Rich ecosystem and tooling
- Familiar to web developers

## Setup

```bash
# Install Node.js dependencies
npm install

# Or with setup script
chmod +x setup.sh
./setup.sh
```

### System Requirements

**Node.js**: v16+ (v18+ recommended for best BigInt performance)

**Installation:**

```bash
# Debian/Ubuntu (via NodeSource)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOS
brew install node
```

## Expected Usage

```bash
# Build TypeScript
npm run build

# Run calculator
npm start -- 500

# Or direct TypeScript execution
npm run dev -- 500
```

## Prototype Tasks

- [ ] Implement parallel primorial + search using worker_threads
- [ ] Use bigint-crypto-utils or similar for primality testing
- [ ] Implement progress reporting
- [ ] Benchmark vs Rust baseline (n=500, n=1000)
- [ ] Evaluate performance gap (native BigInt 10-50x slower expected)
- [ ] Consider WebAssembly + GMP for better performance

## Expected Benchmarks

| n    | F(n) | Time (estimated)     | vs Rust |
| ---- | ---- | -------------------- | ------- |
| 500  | 5167 | TBD (10-50x slower?) | TBD     |
| 1000 | 8719 | TBD (10-50x slower?) | TBD     |

**Note**: Native JavaScript BigInt is 10-50x slower than GMP. WebAssembly + GMP port could close gap to 80-90% native performance.

## Expected Benefits

- **Accessibility**: JavaScript most widely known language
- **Type safety**: TypeScript provides compile-time checking
- **Tooling**: Excellent IDE support, debuggers, formatters
- **Ecosystem**: npm packages for everything
- **Cross-platform**: Runs everywhere Node.js does

## Expected Trade-offs

- **Performance**: Native BigInt significantly slower than GMP
- **Memory**: V8 heap overhead
- **Maturity**: Big integer ecosystem less mature than Python/Rust
- **WASM complexity**: Better performance requires WebAssembly + GMP port

## Dependencies

```json
{
  "dependencies": {
    "bigint-crypto-utils": "^3.3.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "@types/node": "^20.0.0",
    "ts-node": "^10.9.0"
  }
}
```

## Project Structure

```text
node-ts/
├── src/
│   └── fortunate.ts    # Main implementation
├── package.json
├── tsconfig.json
├── README.md
└── setup.sh
```

## References

- BigInt documentation: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt>
- bigint-crypto-utils: <https://github.com/juanelas/bigint-crypto-utils>
- Node.js worker_threads: <https://nodejs.org/api/worker_threads.html>
- Rust baseline: [../rust/](../rust/)

## Performance Considerations

Native BigInt is convenient but slow. For production use, consider:

1. **WebAssembly + GMP**: Compile GMP to WASM for near-native performance
2. **Hybrid approach**: Use Node.js for orchestration, WASM for computation
3. **Accept trade-off**: 10-50x slower but maximum accessibility

## Status

Awaiting prototype implementation. Directory structure ready.
