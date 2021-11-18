target extended-remote :3333

# print demangled symbols
set print asm-demangle on

# disable pagination
set pagination off

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# enable itm
# daisy
#monitor tpiu config internal /tmp/itm.fifo uart off 480000000 2000000
#monitor tpiu config internal /tmp/itm.fifo uart off 200000000 2000000
monitor tpiu config internal /tmp/itm.fifo uart off 96000000 2000000
# ?
#monitor tpiu config internal /tmp/itm.fifo uart off 48000000
# stm32f4
#monitor tpiu config internal /tmp/itm.fifo uart off 168000000
monitor itm port 0 on

# enable semihosting
#monitor arm semihosting enable

load
continue
