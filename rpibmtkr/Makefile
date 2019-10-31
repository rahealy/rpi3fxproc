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
TARGET = aarch64-unknown-none
KERNEL = kernel8.img

#Bootloader
TX_PATH = ../rpi3serbtldr/tx
TX_EXE  = $(TX_PATH)/rpi3serbtldr_tx
TX_DEV  = /dev/ttyACM2
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV) -e

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
	$(MAKE) -C ./drivers clean
	$(MAKE) -C ./hats clean
	$(MAKE) -C ./peripherals clean
	$(MAKE) -C ./startup clean

timer: ./examples/timer.rs ./startup/src/startup.rs
	cargo xrustc --example timer --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/timer ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL)

ultra2: ./examples/ultra2.rs ./startup/src/startup.rs
	cargo xrustc --example ultra2 --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/ultra2 ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL)

i2s: ./examples/i2s.rs ./startup/src/startup.rs
	cargo xrustc --example i2s --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/i2s ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL)

i2s_listener: ./examples/i2s_listener.rs ./startup/src/startup.rs
	cargo xrustc --example i2s_listener --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/i2s_listener ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL)

load:
	$(TX_EXE) $(TX_ARGS) -f ./$(KERNEL)
