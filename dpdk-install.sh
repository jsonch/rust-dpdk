#!/usr/bin/env bash
set -e

# DPDK version to install
# Using 22.11 to match what rust-dpdk was tested with
DPDK_VERSION=${DPDK_VERSION:-22.11}

# Create local dpdk directory
mkdir -p dpdk

# Download DPDK
echo "Downloading DPDK ${DPDK_VERSION}..."
curl -s -o dpdk.tar.xz https://fast.dpdk.org/rel/dpdk-${DPDK_VERSION}.tar.xz

# Extract
echo "Extracting..."
tar -xJf dpdk.tar.xz -C dpdk --strip-components=1

# Build
cd dpdk
echo "Setting up build with meson..."
# Use x86-64 baseline with SSE4.2 and RTM instead of native
# This is compatible with Rosetta x86-64 emulation which doesn't support AVX
export CFLAGS="-march=x86-64 -msse4.2 -mrtm"
meson setup build -Dplatform=generic

echo "Building with ninja..."
ninja -C build

echo "Installing..."
ninja -C build install

echo "Running ldconfig..."
sudo ldconfig

# Cleanup
cd ..
echo "Cleaning up..."
rm -rf dpdk dpdk.tar.xz

echo "DPDK ${DPDK_VERSION} installation complete!"
echo "Note: Configured with -march=x86-64 -msse4.2 -mrtm for Rosetta compatibility"
