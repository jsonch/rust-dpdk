# Rust DPDK Reflector Example

A Rust implementation of a simple DPDK packet reflector that receives packets on a port and sends them back out the same port.

## Building

```bash
cargo build --release
```

## Running with pcap files

```bash
# Create a test pcap file (you can use your own)
# Run the reflector
sudo ../target/release/reflector -l 0 --no-huge --no-pci --vdev 'net_pcap0,rx_pcap=test.pcap,tx_pcap=out.pcap' -- 0
```

## Project Structure

- `src/main.rs` - Main reflector implementation
- `Cargo.toml` - Rust project configuration

This example demonstrates:
- DPDK EAL initialization from Rust
- Port configuration and setup
- Mbuf pool creation
- RX/TX queue setup
- Packet receive and transmit burst operations
- Memory management with mbufs
