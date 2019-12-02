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

KERNEL = kernel8
KERNEL_IMAGE = kernel8.img
TARGET = aarch64-unknown-none-softfloat
BINARY = bootloader_rx

all: debug

debug:
	cargo xrustc --target=$(TARGET)
	cp ./target/$(TARGET)/debug/$(BINARY) ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

release:
	cargo xrustc --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/$(BINARY) ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

clean:
	cargo clean
	-rm $(KERNEL)
	-rm $(KERNEL_IMAGE)
