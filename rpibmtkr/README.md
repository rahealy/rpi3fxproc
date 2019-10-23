# rpibmtkr
Raspberry Pi Bare Metal Toolkit In Rust

## About

Provides an OS-less (bare metal) library to interface with various periperhals on the RPi Broadcom SOC and the Audio Injector Ultra 2 sound card.

This library contains code Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com> and bzt [https://github.com/bztsrc](https://github.com/bztsrc). 

Much of the RPi hardware specific code in this library is derived from information in the "Bare-metal and Operating System development tutorials in Rust on the Raspberry Pi 3" [https://github.com/rust-embedded/rust-raspi3-OS-tutorials](https://github.com/rust-embedded/rust-raspi3-OS-tutorials). This is a recommended resource for anyone interested in learning the specifics involved in solving problems in this particular domain.

This software is pre-alpha. No guarantees.

## Installation

### Dependencies

Install cargo-xbuild and cargo-binutils libraries. These do most of the work to make sure the rust dependencies for generating ARM code for the RPi3 are met.

```
$ cargo install cargo-xbuild cargo-binutils
```

### Bootloader

For development it's recommended to use a serial booloader like "rpi3serbtldr" included in this repository. See the "README.md" in the "rpi3serbtldr" directory for details on installation and use.

### Build

Try:

```
make clean examples
```

Example executables are located in "rpibmtkr" directory.

```
$ ../rpi3serbtldr/tx/rpi3serbtldr_tx -b 115200 -p "/dev/ttyACM0" -f "timer" -t 8000

rpi3serbtldr_tx
---------------
File: timer
Port: "/dev/ttyACM0"
Baud: 115200
Timeout(ms): 8000

Begin...
Receieved break signal.
Sent file size: 1872.
Got OK signal.
Sending file.
Got OK signal.
File sent successfully. Read and echo replies.
Timer1: Begin one second one shot delay.
Timer1: End one second one shot delay.
Timer3: Begin one second one shot delay.
Timer3: End one second one shot delay.
^C
```

