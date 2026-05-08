# ESP32-C6 Rust Thermometer

A software-focused starter project for learning async Rust on the ESP32-C6.

## Tech Stack
- **Chip:** ESP32-C6 (RISC-V)
- **Framework:** [Embassy](https://embassy.dev/) (Async/Await)
- **HAL:** `esp-hal` (with `unstable-hal` features)
- **Networking:** `esp-wifi` (via `esp-radio`)
- **Alloc:** `esp-alloc` (required for WiFi)
- **Logging:** `log` + `esp-backtrace`

## Features
- [ ] Connect to WiFi
- [ ] Read internal temperature sensor
- [ ] Log JSON-formatted health packets to Serial
- [ ] Onboard RGB LED status indication

## Prerequisites
1. Install the RISC-V target:
   ```bash
   rustup target add riscv32imac-unknown-none-elf
   ```
2. Install `espflash`:
   ```bash
   cargo install espflash
   ```

## Running
```bash
cargo run --release
```
