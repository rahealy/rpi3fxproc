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
TARGET = aarch64-unknown-none
KERNEL = kernel8
KERNEL_IMAGE = kernel8.img

#Bootloader
TX_PATH = ./rpi3serbtldr/tx
TX_EXE  = $(TX_PATH)/rpi3serbtldr_tx
TX_DEV  = /dev/ttyACM0
TX_ARGS = -b 115200 -j -t 8000 -p $(TX_DEV) -e


#######################################################################
# Targets
#######################################################################


all:
	cargo xrustc --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/rpi3fxproc ./$(KERNEL)
	cargo objcopy -- --strip-all -O binary ./$(KERNEL) ./$(KERNEL_IMAGE)

clean:
	cargo clean
	-rm -f ./$(KERNEL)
	-rm -f ./$(KERNEL_IMAGE)

realclean: clean
	$(MAKE) -C ./rpi3serbtldr clean
	$(MAKE) -C ./rpibmtkr clean

xrpi3serbtldr:
	$(MAKE) -C ./rpi3serbtldr release

load: xrpi3serbtldr
	$(TX_EXE) $(TX_ARGS) -f ./$(KERNEL_IMAGE)
