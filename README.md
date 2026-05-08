# ESP32-C6 High-Performance Telemetry System

A professional, software-focused template for building Hardware-in-the-Loop (HIL) testing systems for robotics. This project demonstrates a bidirectional binary communication pipeline between an ESP32-C6 microcontroller and a host computer (macOS/Linux).

## System Architecture

The project is organized as a Cargo Workspace to ensure type safety and code reuse across the entire stack.

### 1. `common/` (Shared Protocol)
- **Role:** The "Source of Truth" for communication.
- **Contents:** Defines the `Telemetry` and `Command` data structures.
- **Tech:** Uses `serde` for serialization.

### 2. `firmware/` (The Robot)
- **Role:** Runs on the ESP32-C6 silicon.
- **Contents:** 
  - `rx.rs`: Asynchronously listens for binary commands from the station.
  - `tx.rs`: Streams high-frequency internal temperature telemetry.
- **Tech:** Embassy (Async/Await), `esp-hal`, `postcard` (COBS framing).

### 3. `station/` (The Ground Station)
- **Role:** CLI application running on your Mac.
- **Contents:**
  - `rx.rs`: Decodes the binary telemetry stream into human-readable logs.
  - `tx.rs`: Provides an interactive shell to send commands to the robot live.
- **Tech:** `std` Rust, `serialport`, `postcard`.

## Data Flow (Symmetric RX/TX)

```text
[ STATION (Mac) ]                        [ FIRMWARE (ESP32) ]
       |                                          |
   (tx.rs) -- [ Binary Command ] -------------> (rx.rs)
       |                                          |
   (rx.rs) <- [ Binary Telemetry ] ------------ (tx.rs)
```

## Running the System

1. **Start the Robot:**
   ```bash
   cd firmware
   cargo run --release
   ```
   *(Stop the monitor with Ctrl+C after flashing to free the serial port).*

2. **Start the Station:**
   ```bash
   cd station
   cargo run
   ```

3. **Interact:**
   Press **Enter** in the station terminal to toggle the robot's logging frequency between 1s and 5s live.
