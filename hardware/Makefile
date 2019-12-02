#
# Makefile

# Targets:
#
#  all - Builds debug version.
#  release - Builds optimized release version.
#  clean - Cleans the build directories
#  examples - builds the examples
#
# Example:
#  $ make clean release
#  $ make timer load
#

#######################################################################
# Variables
#######################################################################

#Build
TARGET = aarch64-unknown-none-softfloat
KERNEL = kernel8
KERNEL_IMAGE = kernel8.img

#Bootloader
TX_PATH = ../bootloader/tx
TX_EXE  = $(TX_PATH)/bootloader_tx
TX_DEV  = /dev/ttyACM0
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV) -e

DOCKER_CMD        = docker run -it --rm
DOCKER_ARG_CURDIR = -v $(shell pwd):/work -w /work
DOCKER_ARG_TTY    = --privileged -v /dev:/dev
DOCKER_ARG_NET    = --network host

CONTAINER_GDB     = andrerichter/raspi3-gdb

#######################################################################
# Targets
#######################################################################

all: debug

debug:
	cargo xrustc --target=$(TARGET)

release:
	cargo xrustc --target=$(TARGET) --release -- --emit asm

clean:
	cargo clean
	-rm -f ./$(KERNEL)
	-rm -f ./$(KERNEL_IMAGE)
	$(MAKE) -C ./drivers clean
	$(MAKE) -C ./hats clean
	$(MAKE) -C ./peripherals clean
	$(MAKE) -C ./startup clean

timer: ./examples/timer.rs ./startup/src/startup.rs
	cargo xrustc --example timer --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/examples/timer ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

exception: ./examples/exception.rs ./startup/src/startup.rs
	cargo xrustc --example exception --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/examples/exception ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

load:
	$(TX_EXE) $(TX_ARGS) -f ./$(KERNEL_IMAGE)
