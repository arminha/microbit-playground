# micro:bit playground

Playing with the BBC micro:bit

## Documentation

* [micro:bit Hardware Documentation](https://tech.microbit.org/hardware/1-5-revision/)
* [MicroRust](https://droogmic.github.io/microrust/) book.

### Helpful commands

Start an OpenOCD connection

```sh
openocd -f interface/cmsis-dap.cfg -f target/nrf51.cfg -l /tmp/openocd.log
```

Start minicom

```sh
minicom -D /dev/ttyACM0 minirc.dfl
```
