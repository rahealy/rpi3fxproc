#
# Makefile

# Targets:
#
#  all - Builds debug version.
#  release - Builds optimized release version.
#  clean - Cleans the build directories
#
# Example:
#  $ make clean release
#

#######################################################################
# Variables
#######################################################################

#Build
# TARGET = aarch64-unknown-none
TARGET = aarch64-unknown-none-softfloat
KERNEL = kernel8
KERNEL_IMAGE = kernel8.img

#Bootloader
TX_PATH = ./bootloader/tx
TX_EXE  = $(TX_PATH)/bootloader_tx
TX_DEV  = /dev/ttyACM0
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV) -e


#######################################################################
# Targets
#######################################################################


all: _bootloader
	cargo xrustc --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/rpi3fxproc ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

debug:
	cargo xrustc --target=$(TARGET)
	cp ./target/$(TARGET)/debug/rpi3fxproc ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

clean:
	cargo clean
	-rm -f ./$(KERNEL)
	-rm -f ./$(KERNEL_IMAGE)

realclean: clean
	$(MAKE) -C ./bootloader clean
	$(MAKE) -C ./common clean
	$(MAKE) -C ./effects clean
	$(MAKE) -C ./hardware clean

_bootloader:
	$(MAKE) -C ./bootloader release

load: _bootloader
	$(TX_EXE) $(TX_ARGS) -f ./$(KERNEL_IMAGE)
