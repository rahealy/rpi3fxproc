# rpi3fxproc
## Raspberry Pi 3 Bare Metal Effects Processor. 

Implements a bare metal OS-less full-duplex audio effects processor using a Raspberry Pi 3 embedded computer and an Audio Injector Ultra 2 Sound Card.

<img src="dev_setup.jpg" alt="Current development setup." height="423" width="640"/>

## Story

Building an audio effects processor has been on my (rahealy's) bucket-list for decades. After quitting the software support job I was working to pay the bills I decided to take a vacation and pursue my true passion and calling in life. In that time I learned another programming language (rust), learned some of the advanced internals of ARMv8, AARCH64, Execution priviledges, how MMU's work, multi-core CPUs, as well as getting a good start on building an audio effects processor. 

Please feel free to take or leave what you find here. Be good, follow the licenses, and give credit where it's due. Also please feel free to open an issue if you have any questions. I am happy to (over)share what I've learned.

## Status

Currently alpha.

(12/02/2019) Alpha release provides most of the framework for effects processing graph and demonstrates a 2 second delay. Continue working on To-Do list.


## Credits

### Software

This project contains code Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com> and bzt [https://github.com/bztsrc](https://github.com/bztsrc). All copyrights are respective of their owners.

Much of the RPi hardware specific code in this project is derived from information in the "Bare-metal and Operating System development tutorials in Rust on the Raspberry Pi 3" [https://github.com/rust-embedded/rust-raspi3-OS-tutorials](https://github.com/rust-embedded/rust-raspi3-OS-tutorials). This is a recommended resource for anyone interested in learning the specifics involved in solving problems in this particular domain.

### Hardware

Details about the Audio Injector Ultra 2 sound card by flatmax:

[https://github.com/flatmax](https://github.com/flatmax)

[http://www.audioinjector.net/rpi-ultra](http://www.audioinjector.net/rpi-ultra)

[https://www.amazon.com/Audio-Injector-Ultra-Sound-Raspberry/dp/B07HQ8FFPJ/](https://www.amazon.com/Audio-Injector-Ultra-Sound-Raspberry/dp/B07HQ8FFPJ/)


## Build and Installation

### Dependencies

Install cargo-xbuild and cargo-binutils libraries. These do most of the work to make sure the rust dependencies for generating ARM code for the RPi3 are met.

```
$ cargo install cargo-xbuild cargo-binutils
```

### Bootloader

For development it's recommended to use a serial booloader. A bootloader and client  is included in the `bootloader` sub-directory. See the relevant "README.md" for details on installation and use.

### Build

After reading the various README.md files, installing the dependencies and bootloader change to the to the project root directory and run:

```
$ make all
```

If everything goes well there should be a `kernel8` and `kernel8.img` file in the root directory. 

### Install

**Install Via SDCard**

Copy kernel8.img to the ./boot directory of a suitably prepared SDCard. Insert card into Raspberry Pi 3 and boot.


**Install Via Bootloader**

The makefile `load` target uses the bootloader client to send kernel.img to the hardware. Install the bootloader on the Raspberry Pi 3 per the bootloader README. Verify the configuration in the makefile `#Bootloader` section is correct then run:

```
make load
```

Output should look something like this:

```
./bootloader/tx/bootloader_tx -b 115200 -j -t 8000 -p /dev/ttyACM0 -e -f ./kernel8.img

rpi3fxproc bootloader_tx
------------------------
File: ./kernel8.img
Port: "/dev/ttyACM0"
Baud: 115200
Timeout(ms): 8000
JTAG: Yes
Wait: No
Read and Echo: Yes

Begin...
Received break signal.
Send JTAG instruction.
Got OK signal.
Sent file size: 65152.
Got OK signal.
Sending file.
Got OK signal.
Send JUMP instruction.
bootloader_tx - Read port and echo output.
```

## Alpha Release To-Do

### Integrate FFT math

Integrate local `#[no_std]` port of RustFFT for tone controls.

### DMA

Learn how to enable and use i2s DMA.

### Open Sound Control (OSC)

Integrate OSC support.

### Multi-Core

Tentative split:

* Core 0 - UART, UI, OSC
* Core 1 - Effects rack unit 0
* Core 2 - Effects rack unit 1
* Core 3 - Software synth (un-tiss un-tiss beeeowwwoop)
