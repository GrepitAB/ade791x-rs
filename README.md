# Rust ADE791x 3-Channel, Isolated, Sigma-Delta ADC with SPI Driver 

This is a platform-agnostic Rust driver for the ADE7912/ADE7913 3-Channel, Isolated, Sigma-Delta ADC with SPI, using the [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) traits.

This driver allows you to:

- Initialize and configure the device.
- Perform a hardware/software reset.
- Powerdown/wakeup the device.
- Get raw and converted measurements from the ADC.
- Manage multiple ADCs configured in a polyphase metering system (see the `poly` module).

## The devices

The ADE7912/ADE7913 are isolated, 3-channel Σ-Δ ADCs for polyphase energy metering applications using shunt current sensors. Data and power isolation are based on the Analog Devices, Inc., [*i*Coupler® Technology](https://www.analog.com/en/products/landing-pages/001/icoupler-technology-alternative-to-optocouplers.html). The ADE7912 features two ADCs, and the ADE7913 features three ADCs. The current ADC provides a 67 dB Signal-to-Noise Ratio (SNR) over a 3 kHz signal bandwidth, whereas the voltage ADCs provide an SNR of 72 dB over the same bandwidth. One channel is dedicated to measuring the voltage across a shunt when the shunt is used for current sensing. Up to two additional channels are dedicated to measuring voltages, which are usually sensed using resistor dividers. One voltage channel can measure the temperature of the die via an internal sensor. The ADE7913 includes three channels: one current and two voltage channels. The ADE7912 has one voltage channel but is otherwise identical to the ADE7913.

##### Datasheets:

- [ADE7912/ADE7913 (Rev. C)](https://www.analog.com/media/en/technical-documentation/data-sheets/ade7912_7913.pdf)

## Usage

First of all, in order to get correct readings, the ADC needs to be calibrated according to the following procedure:

1. Set the calibration offsets and multipliers to their default values.
2. Remove any load from the ADC.
3. Calculate the offsets as the average of the ADC readings with no load applied.
4. Set the calculated calibration offsets, leaving the multipliers to their default value.
5. Apply a known load to the ADC.
6. Calculate the multipliers by dividing the known load by the average of the ADC readings with the known load applied.

The followings are two minimal examples to get readings from the ADC, both in a single-phase ADC configuration and a poly-phase multi ADCs configuration.

### Single

```rust
use ade791x::*;

// Initialization
let config = Config::default();
let calibration = Calibration::default();
let mut adc = Ade791x::new_ade7912(spi, cs);
adc.init(delay, config, calibration).unwrap();

// Measurement
// Run the following in the DREADY ISR to get measurements as soon as they are ready
let measurement = adc.get_measurement().unwrap();
```

### Poly

```rust
use ade791x::*;

// Initialization
let config = [
    Config { clkout_en: true, ..Default::default() },
    Config { clkout_en: true, ..Default::default() },
    Config::default()
];
let calibration = [Calibration::default(); 3];
let emi_ctrl = [
    ade791x::EmiCtrl::from(0x55),
    ade791x::EmiCtrl::from(0xAA),
    ade791x::EmiCtrl::from(0x55)
];
let mut adc = poly::Ade791x::new(spi, [
    (cs0, Chip::ADE7912), (cs1, Chip::ADE7913), (cs2, Chip::ADE7912)
]);
adc.init(delay, config, calibration, emi_ctrl).unwrap();

// Synchronization
// Execute the following every couple of seconds to ensure that the ADCs are always in sync
adc.ajust_sync().unwrap();

// Measurement
// Run the following in the DREADY ISR to get measurements as soon as they are ready
let measurement = adc.get_measurement().unwrap();
```

## Status

- [x] Initialization/configuration
- [x] Hardware/software reset
- [x] Powerdown/wakeup
- [x] Measurement reading
- [x] Polyphase management
- [x] Measurement conversion
- [x] Temperature readings
- [x] Configuration checks
- [x] Unit tests
- [ ] Measurement CRC checks

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
