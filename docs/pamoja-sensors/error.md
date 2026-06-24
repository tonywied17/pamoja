# pamoja-sensors::error

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The error type shared by the sensor drivers.

## enum `SensorError`

What can go wrong turning a part's raw bytes into a reading.

Most of these drivers only ever decode well-formed register values and so cannot
fail, but parts that carry their own integrity check report a mismatch here so the
caller re-reads rather than trusting corrupted data.

- `Crc` - A device's own checksum did not match the bytes it covered, so the read was corrupted on the bus and must be repeated. Returned, for example, when a DS18B20 scratchpad's CRC byte disagrees with the data bytes.

