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

TARGET = aarch64-unknown-none
EXAMPLES = timer ultra2 i2s i2s_listener

all: debug

debug:
	cargo xrustc --target=$(TARGET)

release:
	cargo xrustc --target=$(TARGET) --release

clean:
	cargo clean
	-rm -f $(EXAMPLES)

examples: $(EXAMPLES)

timer: ./examples/timer.rs ./examples/startup.rs
	cargo xrustc --example timer --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/examples/timer .
	cargo objcopy -- --strip-all -O binary ./timer

ultra2: ./examples/ultra2.rs ./examples/startup.rs
	cargo xrustc --example ultra2 --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/examples/ultra2 .
	cargo objcopy -- --strip-all -O binary ./ultra2

i2s: ./examples/i2s.rs ./examples/startup.rs
	cargo xrustc --example i2s --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/examples/i2s .
	cargo objcopy -- --strip-all -O binary ./i2s

i2s_listener: ./examples/i2s_listener.rs ./examples/startup.rs
	cargo xrustc --example i2s_listener --target=$(TARGET) --release
	cp ./target/$(TARGET)/release/examples/i2s_listener .
	cargo objcopy -- --strip-all -O binary ./i2s_listener
