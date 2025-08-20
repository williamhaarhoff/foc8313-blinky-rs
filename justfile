build:
    cargo build

blinky:
    cargo run --bin blinky

printy:
    cargo run --bin printy

erase:
    openocd -f interface/stlink.cfg -f target/stm32f1x.cfg -c "reset_config srst_only srst_nogate connect_assert_srst" -c "init" -c "reset halt;" -c 'flash erase_sector 0 0 last; reset' -c 'shutdown'

erase-brute-force:
    while [ 1 ]; do just erase && break; done    

serial:
    picocom /dev/ttyUSB0 --imap lfcrlf

can_monitor:
    sudo ip link set down can0 && sudo ip link set up can0 type can bitrate 1000000 && candump can0
