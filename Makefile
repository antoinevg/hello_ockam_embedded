clean:
	cargo clean

itm:
	rm -f /tmp/itm.fifo
	touch /tmp/itm.fifo
	itmdump -F -f /tmp/itm.fifo

std:
	cargo build --example $(example)
	leaks --atExit -- target/debug/examples/$(example)

no_std:
	cargo build --example $(example) --no-default-features --features="alloc, no_std"
	leaks --atExit -- target/debug/examples/$(example)

qemu:
	cargo +nightly run --example $(example) --target thumbv7em-none-eabihf --no-default-features --features="qemu"


# 2048 kB Flash, 512 kB RAM
nucleo-h7xx:
	cp memory-nucleo-h7xx.x	memory.x
	cargo +nightly -Z unstable-options \
		--config "target.'cfg(target_arch=\"arm\")'.runner = 'arm-none-eabi-gdb -q -x openocd-itm.gdb'" \
		run --example $(example) \
			--release \
			--target thumbv7em-none-eabihf \
			--no-default-features \
			--features="stm32h7,bsp_nucleo_h7xx"

# 1024 kB Flash, 256 kB RAM
atsame54:
	cp memory-atsame54.x memory.x
	cargo +nightly -Z unstable-options \
		--config "target.'cfg(target_arch=\"arm\")'.runner = 'arm-none-eabi-gdb -q -x openocd-uart.gdb'" \
		run --example $(example) \
			--release \
			--target thumbv7em-none-eabihf \
			--no-default-features \
			--features="atsame54"

# --config "target.'cfg(target_arch=\"arm\")'.runner = './xpacks/.bin/arm-none-eabi-gdb -q -x openocd-itm.gdb'"


stm32f4:
	cargo +nightly run --example $(example) --release --target thumbv7em-none-eabihf --no-default-features --features="stm32f4"

daisy:
	cargo +nightly run --example $(example) --release --target thumbv7em-none-eabihf --no-default-features --features="stm32h7, daisy"

wasm:
	cargo build --target=wasm32-unknown-unknown --no-default-features --features="alloc, no_std"
