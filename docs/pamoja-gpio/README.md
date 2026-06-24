# pamoja-gpio

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

On-board bus addressing and pin logic for the pamoja SDK.

Before a node reaches any network it has to talk to the chips wired to the board it
sits on. Three interfaces cover almost everything cheap hardware uses: I2C for the
dense breakout sensors (a BME280, an INA226, an MPU9250, an SSD1306 screen), SPI for
displays, SD cards, and LoRa radios, and plain GPIO pins for the relays, valves,
buttons, and motion sensors that switch a single line. Each carries a small, exact
piece of logic that is pure arithmetic with no hardware attached, and getting it wrong
is a classic field bug: the wrong I2C address byte, the wrong SPI clock mode, or an
active-low relay driven as if it were active-high.

This crate is that logic, with no pins toggled and no allocation:

- [`i2c`] - I2C addressing per the NXP I2C-bus specification (UM10204): the 7-bit
  address byte `(address << 1) | r/w`, the two-byte `11110xx` frame for a 10-bit
  address, and the reserved ranges (`0x00..=0x07` and `0x78..=0x7F`) that leave
  `0x08..=0x77` for real devices, so a bad address is caught before it reaches the bus.
- [`spi`] - the four SPI clock [`Mode`](spi::Mode)s as the `(CPOL, CPHA)` pair every
  datasheet quotes, plus bit order, so "mode 3, MSB first" is a checked value rather
  than two booleans a caller can transpose.
- [`pin`] - the GPIO pin model: physical [`Level`](pin::Level), input pull and output
  drive, the interrupt [`Edge`](pin::Edge), and an active-high/active-low
  [`Polarity`](pin::Polarity) that maps a logical "asserted" onto the physical level,
  so an active-low button or relay is handled by the type rather than by remembering
  to invert.

Everything is exact integer work over `Copy` values, so the same logic runs on the
smallest microcontroller driving the bus. Clocking the bytes and toggling the lines
themselves arrives with the hardware-I/O layer; this is the addressing-and-mode half
ahead of it.

**Examples**

```
use pamoja_gpio::i2c::{Address, Direction};
use pamoja_gpio::pin::{Level, Polarity};
use pamoja_gpio::spi::Mode;

// A DS3231 real-time clock answers at 7-bit address 0x68; its read frame is one byte.
let rtc = Address::seven_bit(0x68)?;
let mut frame = [0u8; 2];
let n = rtc.write_frame(Direction::Read, &mut frame)?;
assert_eq!(&frame[..n], &[0xD1]); // (0x68 << 1) | 1

// SPI clock mode 0 is the (CPOL, CPHA) pair (false, false), as a datasheet quotes it.
assert_eq!(Mode::Mode0.cpol_cpha(), (false, false));

// An active-low relay is energised by driving its pin low.
assert_eq!(Polarity::ActiveLow.level(true), Level::Low);
```

## Modules

- [error](error.md)
- [i2c](i2c.md)
- [pin](pin.md)
- [spi](spi.md)

