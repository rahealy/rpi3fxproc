#
# Makefile
#  Rust cargo doesn't quite have the functionality we need yet so using
#  a very simple makefile to automate building.
#
# Targets:
#
#  all - Builds debug version.
#  release - Builds optimized release version.
#  clean - Cleans the build directories
#
# Example:
#  $ make clean release
#

TARGET = aarch64-unknown-none
BINARY = rpi3serbtldr_rx

all: debug

debug:
	cargo xrustc --target=$(TARGET) -- --emit asm
	cp ./target/$(TARGET)/debug/$(BINARY) ./kernel8.img
	cargo objcopy -- --strip-all -O binary ./kernel8.img

release:
	cargo xrustc --target=$(TARGET) --release -- --emit asm
	cp ./target/$(TARGET)/release/$(BINARY) ./kernel8.img
	cargo objcopy -- --strip-all -O binary ./kernel8.img

clean:
	cargo clean
	-rm $(BINARY)
	-rm ./kernel8.img
