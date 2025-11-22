# tatted

This is a userspace driver for the JD79668 4-color 4.2" e-ink display controller. This is the controller used in the Pimoroni wHAT (wide hat) designed for the Raspberry Pi. This is a workspace crate which includes
* `libtatted` the high-level driver library, including image pre-processing
* `tatctl` a small CLI for controlling the display, rendering images, hardware resets, etc.

All my testing of this crate has been using a Raspberry Pi 4 Model B running NixOS rather than Raspberry Pi OS, my NixOS configuration for the pi can be found [here](https://github.com/treyfortmuller/pi-nixos).

I built this thing with help from a now-archived Rust port of the Pimoroni [inky](https://github.com/pimoroni/inky) python library authored by Axel Örn Sigurðsson called [`paperwave`](https://crates.io/crates/paperwave). 

The datasheet/manual for the JD79668 can be found [here](https://files.waveshare.com/wiki/4.2inch%20e-Paper%20Module%20(G)/4.2inch_e-Paper_(G).pdf).




