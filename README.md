# hello_ockam_embedded

## Install dependencies

    rustup toolchain install nightly
    rustup target add thumbv7em-none-eabihf --toolchain nightly
    brew install qemu

## Running examples

### 01-node

    make example=01-node std
    make example=01-node no_std
    make example=01-node qemu
    make example=01-node atsame54
    make example=01-node stm32f4
    make example=01-node daisy
    make example=01-node nucleo-h7xx


## Monitoring debug output

For itm:

    itmdump -F -f /tmp/itm.fifo

For uart:

    picocom -b 115200 --imap lfcrlf /dev/tty.usbserial-AR0K4XGV

## BLE Transport

The BLE examples all use a STEVAL-IDB005V1D module.

Also see:

* datasheets.git/STEVAL-IDB005V1D/um1870-bluenrgms-development-kits-stmicroelectronics.pdf - page 8, 9
* datasheets.git/ATSAME54-XPRO/user-guide.pdf - 4.5.1 Xplained Pro Standard Extension Header
* https://github.com/atsamd-rs/atsamd/blob/master/boards/atsame54_xpro/src/pins.rs
* https://www.oshwa.org/a-resolution-to-redefine-spi-signal-names/


### STEVAL-IDB005V1D

```
IDB005V1D

Looking at bottom of board with antenna at top, J4 on left, J3 right:

    J4-09  MISO --- [SDO]      brown          J4-10  TEST9
    J4-07  MOSI --- [SDI]      purple         J4-08  TEST1
    J4-05  CLK ---- [SPI_SCK]  blue           J4-06  NC
    J4-03  CSN ---- [SPI_SS_A] orange         J4-04  IRQ ---- [IRQ7] yellow
    J4-01  GND                                J4-02  GND


    J3-09  RST --- [GPIO1]     green          J3-10  VDD ---- [VCC]  red
    J3-07  GND --- [GND]       black          J3-08  VDD ---- [VCC] -+
    J3-05  GND                                J3-06  nS              |
    J3-03  NC                                 J3-04  NC              |
    J3-01  NC                                 J3-02  3V3 ---- [VCC] -+
```

### ATSAME54XP

```
ATSAME54XP

Looking at top of board with USB DEBUG on top and EXT1 on right:

    EXT2-01  ID                               EXT2-02  GND
    EXT2-03  PB00                             EXT2-04  PA03
    EXT2-05  PB01 GPIO1 ----- [RST] green     EXT2-06  PB06 GPIO2
    EXT2-07  PB14                             EXT2-08  PB15
    EXT2-09  PD00 IRQ0 ------ [IRQ] yellow    EXT2-10  PB02 SPI_SS_B
    EXT2-11  PD08                             EXT2-12  PD09
    EXT2-13  PB17 USART_RX                    EXT2-14  PB16 USART_TX
    EXT2-15  PC06 SPI_SS_A -- [CSN] orange    EXT2-16  PC04 SPI_MOSI --- [SDI] purple
    EXT2-17  PC07 SPI_MISO -- [SDO] brown     EXT2-18  PC05 SPI_SCK ---- [CLK] blue
    EXT2-19  GND ------------ [GND] black     EXT2-20  VCC ------------- [3V3] red
```

### Nucleo-H745

```
Nucleo-H745

Looking at top of board with USB on top:

    [ - CN8 -------------------------------------------------- ]

     1  NC                                       2  D43 [PC8]  ------------ [IRQ] yellow
     3  IOREF                                    4  D44 [PC9]  ------------ [RST] green
     5  RESET                                    6  D45 [PC10] SPI3_SCK --- [CLK] blue
     7  +3V3 ------------------ [3V3] red        8  D46 [PC11] SPI3_MISO -- [SDO] brown
     9  +5V                                     10  D47 [PC12] SPI3_MOSI -- [SDI] purple
    11  GND  ------------------ [GND] black     12  D48 [PD2]
    13  GND                                     12  D49 [PG10]
    14  VIN                                     12  D50 [PG8]

    [ - CN7 -------------------------------------------------- ]

     1  D16 [PC6]                                2  D15 [PB8]
     3  D17 [PB15]                               4  D14 [PB9]
     5  D18 [PB13]                               6  AVDD
     7  D19 [PB12]                               8  GND
     9  D20 [PA15] SPI3_NSS --- [CSN] orange    10  D13 [PA5]
    11  D21 [PC7]                               12  D12 [PA6]
    13  D22 [PB5]  SPI3_MOSI -- [SDI] purple    14  D11 [PB5]
    15  D23 [PB3]  SPI3_SCK --- [CLK] blue      16  D10 [PD14]
    17  D24 [PA4]  SPI3_NSS --- [CSN] orange    18  D9  [PD15] --- [IRQ] yellow
    19  D25 [PB4]  SPI3_MISO -- [SDO] brown     20  D8  [PG9]  --- [RST] green
```

### Daisy Seed

```
Daisy Seed - https://www.electro-smith.com/daisy/daisy

Looking at top of board with USB on bottom:

    P30  [23]                                 P11  [10] SPI1 MOSI -- [SDI] purple
    P31  [24]                                 P10  [09] SPI1 MISO -- [SDO] brown
    P32  [25]                                 P09  [08] SPI1 SCK --- [CLK] blue
    P33  [26]                                 P08  [07] SPI1 CS ---- [CSN] orange
    P34  [27]                                 P07  [06] GPIOC12 ---- [IRQ] yellow
    P35  [28]                                 P07  [05] GPIOD2 ----- [RST] green
    ...                                       ...
    P38  3V3 ------------- [3V3] red          P03  [02]
    P39  VIN                                  P02  [01]
    P40  GND ------------- [GND] black        P01  [00]
```

### STM32F4-DISCOVERY

```
STM32F4-DISCOVERY

Looking at top of board with USB on top:

    P1-GND --------------- [GND] black        P1-GND
    P1-5V                                     P1-5V
    P1-3V ---------------- [3V3] red          P1-3V
    ...

    ...                                       ...
    P2-PD01                                   P2-PD02  GPIOD2 ------ [RST] green
    P2-PC12  SPI3_MOSI --- [SDI] purple       P2-PD00  GPIOD0 ------ [IRQ] yellow
    P2-PC10  SPI3_SCK ---- [CLK] blue         P2-PC11  SPI3_MISO --- [SDO] brown
    P2-PA14                                   P2-PA15  SPI3_NSS ---- [CSN] orange?
    ...                                       ...

```

### STM32F3-DISCOVERY

```
STM32F3-DISCOVERY

Looking at top of board with USB on top:

    P1-3V ---------------- [3V3] red          P1-3V
    P1-GND --------------- [GND] black        P1-NRST
    ...

    ...                                       ...
    P2-PD01                                   P2-PD02  GPIOD2  ----- [RST] green
    P2-PC12  SPI3_MOSI --- [SDI] purple       P2-PD00  IRQ? -------- [IRQ] yellow
    P2-PC10  SPI3_SCK ---- [CLK] blue         P2-PC11  SPI3_MISO --- [SDO] brown
    P2-PA14                                   P2-PA15  SPI3_NSS ---- [CSN] orange
    ...                                       ...

```


### PIC32MX170F256B

```
PIC32MX170F256B

Looking at top of chip with notch on top:

    01          28 [AVdd] -------------- [3V3] red
    02          27 [AVss] -------------- [GND] black
    03          26
    04          25 [RPB14] SCK1 -------- [CLK] blue
    05          24
    06          23
    07          22 [RPB11] ------------- [RST] green  <- config as input
    08          21 [RPB10] ------------- [IRQ] yellow -> config as output
    09          20
    10          19
    11          18
    12          17 [RPB8.0100] SDI1 ---- [SDO] brown
    13          16 [RPB7.0100] SS1 ----- [CSN] orange ????
    14          15 [RPB6.0011] SDO1 ---- [SDI] purple


UART for debug output is on pin 10

```
