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
#

#######################################################################
# Variables
#######################################################################

#Build
TARGET = aarch64-unknown-none
EXAMPLES = timer ultra2 i2s i2s_listener

#Bootloader
TX_PATH = /home/rhealy/projects/rust_embedded/rpi3fxproc/rpi3serbtldr/tx
TX_EXE  = $(TX_PATH)/rpi3serbtldr_tx
TX_DEV  = /dev/ttyACM0
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV)

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
	-rm -f $(EXAMPLES)

timer: ./examples/timer.rs ./startup/src/startup.rs
	cargo xrustc --example timer --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/timer ./kernel8.img

ultra2: ./examples/ultra2.rs ./startup/src/startup.rs
	cargo xrustc --example ultra2 --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/ultra2 ./kernel8.img

i2s: ./examples/i2s.rs ./startup/src/startup.rs
	cargo xrustc --example i2s --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/i2s ./kernel8.img

i2s_listener: ./examples/i2s_listener.rs ./startup/src/startup.rs
	cargo xrustc --example i2s_listener --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/examples/i2s_listener ./kernel8.img

load:
	cargo objcopy -- --strip-all -O binary ./kernel8.img
	$(TX_EXE) $(TX_ARGS) -f ./kernel8.img 
