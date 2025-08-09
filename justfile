build:
    cargo build

flash:
    cargo run

erase:
    openocd -f interface/stlink.cfg -f target/stm32f1x.cfg -c 'init' -c 'halt; flash erase_sector 0 0 last; reset' -c 'shutdown'

erase-brute-force:
    while [ 1 ]; do just erase && break; done    