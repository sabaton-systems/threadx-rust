# Rust bindings for ThreadX

This crate provides the Rust bindings for the ThreadX RTOS. 

## Supported Targets

1. thumbv6m-none-eabi    # Cortex-M0 and Cortex-M0+
2. thumbv7m-none-eabi    # Cortex-M3
3. thumbv7em-none-eabi   # Cortex-M4 and Cortex-M7 (no FPU)
4. thumbv7em-none-eabihf # Cortex-M4F and Cortex-M7F (with FPU)
5. thumbv8m.base-none-eabi   # Cortex-M23
6. thumbv8m.main-none-eabi   # Cortex-M33 (no FPU)
7. thumbv8m.main-none-eabihf # Cortex-M33 (with FPU)

Building for one of the above targets will select the right ThreadX build configuration
for the target.

## Pre-requisites

gcc-arm-none-eabi  must be installed on your system. 

```sudo apt install gcc-arm-none-eabi```

## This will only give you the ThreadX static library

Your application must include the following files as described in https://learn.microsoft.com/en-us/azure/rtos/threadx/chapter2

1. xxx_crt0.S
2. xxx_vectors.S
3. tx_initialize_low_level.S

Check out the threadx-rs and threadx-app folders to see how to make a full fledged ThreadX application in Rust.

## TX USER CONFIGURATION

Set the TX_USER_FILE environment variable to point to the specific configuration for the ThreadX build.

