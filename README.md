# talk-rustlondon22

## Install dependencies

    rustup target add thumbv7em-none-eabihf --toolchain nightly
    rustup target add wasm32-unknown-unknown
    brew install qemu

## 01-node

    make example=01-node std
    make example=01-node no_std
    make example=01-node qemu
    make example=01-node atsame54
    make example=01-node stm32f4
    make example=01-node daisy

## hello

    make example=hello std
    make example=hello no_std
    make example=hello qemu
    make example=hello atsame54
    make example=hello stm32f4
    make example=hello daisy
