//! # Rust ADE791x 3-Channel, Isolated, Sigma-Delta ADC with SPI Driver
//!
//! This is a platform-agnostic Rust driver for the ADE7912/ADE7913 3-Channel, Isolated, Sigma-Delta
//! ADC with SPI, using the [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) traits.
//!
//! This driver allows you to:
//!
//! - Initialize and configure the device.
//! - Perform a hardware/software reset.
//! - Powerdown/wakeup the device.
//! - Get raw and converted measurements from the ADC.
//! - Manage multiple ADCs configured in a polyphase metering system (see the `poly` module).
//!
//! ## The devices
//!
//! The ADE7912/ADE7913 are isolated, 3-channel Σ-Δ ADCs for polyphase energy metering applications
//! using shunt current sensors. Data and power isolation are based on the Analog Devices, Inc.,
//! [*i*Coupler® Technology](https://www.analog.com/en/products/landing-pages/001/icoupler-technology-alternative-to-optocouplers.html).
//! The ADE7912 features two ADCs, and the ADE7913 features three ADCs. The current ADC provides a
//! 67 dB Signal-to-Noise Ratio (SNR) over a 3 kHz signal bandwidth, whereas the voltage ADCs
//! provide an SNR of 72 dB over the same bandwidth. One channel is dedicated to measuring the
//! voltage across a shunt when the shunt is used for current sensing. Up to two additional channels
//! are dedicated to measuring voltages, which are usually sensed using resistor dividers. One
//! voltage channel can measure the temperature of the die via an internal sensor. The ADE7913
//! includes three channels: one current and two voltage channels. The ADE7912 has one voltage
//! channel but is otherwise identical to the ADE7913.
//!
//! ##### Datasheets:
//!
//! - [ADE7912/ADE7913 (Rev. C)](https://www.analog.com/media/en/technical-documentation/data-sheets/ade7912_7913.pdf)
//!
//! ## Usage
//!
//! First of all, in order to get correct readings, the ADC needs to be calibrated according to the
//! following procedure:
//!
//! 1. Set the calibration offsets and multipliers to their default values.
//! 2. Remove any load from the ADC.
//! 3. Calculate the offsets as the average of the ADC readings with no load applied.
//! 4. Set the calculated calibration offsets, leaving the multipliers to their default value.
//! 5. Apply a known load to the ADC.
//! 6. Calculate the multipliers by dividing the known load by the average of the ADC readings with
//! the known load applied.
//!
//! The followings are two minimal examples to get readings from the ADC, both in a single-phase ADC
//! configuration and a poly-phase multi ADCs configuration.
//!
//! ### Single
//!
//! ```ignore
//! use ade791x::*;
//!
//! // Initialization
//! let config = Config::default();
//! let calibration = Calibration::default();
//! let mut adc = Ade791x::new_ade7912(spi, cs);
//! adc.init(delay, config, calibration).unwrap();
//!
//! // Measurement
//! // Run the following in the DREADY ISR to get measurements as soon as they are ready
//! let measurement = adc.get_measurement().unwrap();
//! ```
//!
//! ### Poly
//!
//! ```ignore
//! use ade791x::*;
//!
//! // Initialization
//! let config = [
//!     Config { clkout_en: true, ..Default::default() },
//!     Config { clkout_en: true, ..Default::default() },
//!     Config::default()
//! ];
//! let calibration = [Calibration::default(); 3];
//! let emi_ctrl = [
//!     ade791x::EmiCtrl::from(0x55),
//!     ade791x::EmiCtrl::from(0xAA),
//!     ade791x::EmiCtrl::from(0x55)
//! ];
//! let mut adc = poly::Ade791x::new(spi, [
//!     (cs0, Chip::ADE7912), (cs1, Chip::ADE7913), (cs2, Chip::ADE7912)
//! ]);
//! adc.init(delay, config, calibration, emi_ctrl).unwrap();
//!
//! // Synchronization
//! // Execute the following every couple of seconds to ensure that the ADCs are always in sync
//! adc.ajust_sync().unwrap();
//!
//! // Measurement
//! // Run the following in the DREADY ISR to get measurements as soon as they are ready
//! let measurement = adc.get_measurement().unwrap();
//! ```
//!

#![no_std]

use core::marker::PhantomData;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

pub use register::*;

pub mod poly;
mod register;
mod ade791x;

/// Represents a single ADE7912/ADE7913 3-Channel, Isolated, Sigma-Delta ADC with SPI.
pub struct Ade791x<SPI, CS> {
    adc: poly::Ade791x<SPI, CS, 1>
}

impl<SPI, CS, S, P> Ade791x<SPI, CS>
    where
        SPI: spi::Transfer<u8, Error=S>,
        CS: OutputPin<Error=P> {

    /// Creates a new [`Ade791x`] instance representing a ADE7912 chip, given the SPI peripheral and
    /// the CS output pin. The newly created instance must be initialized using [`Self::init()`].
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `cs` - The CS output pin implementing the [`OutputPin`] trait.
    pub fn new_ade7912(spi: SPI, cs: CS) -> Self {
        Self {
            adc: poly::Ade791x::new(spi, [(cs, Chip::ADE7912)])
        }
    }

    /// Creates a new [`Ade791x`] instance representing a ADE7913 chip, given the SPI peripheral and
    /// the CS output pin. The newly created instance must be initialized using [`Self::init()`].
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `cs` - The CS output pin implementing the [`OutputPin`] trait.
    pub fn new_ade7913(spi: SPI, cs: CS) -> Self {
        Self {
            adc: poly::Ade791x::new(spi, [(cs, Chip::ADE7913)])
        }
    }

    /// Initializes the ADC, applying the given configuration. After this method, the ADC is ready
    /// to use.
    /// # Arguments
    /// * `delay` - The delay source implementing the [`DelayMs`] trait.
    /// * `config` - The [`Config`] struct containing the configuration for the ADC.
    /// * `calibration` - The [`Calibration`] struct containing the calibration values for the ADC.
    pub fn init(&mut self,
                delay: &mut dyn DelayMs<u32>,
                config: Config,
                calibration: Calibration) -> Result<(), Error<S, P>> {
        self.adc.init(delay, [config], [calibration], [EmiCtrl::default()])
    }

    /// Performs a hardware reset of the ADC. During a hardware reset, all the registers are set to
    /// their default values and the dc-to-dc converter is shut down. After a hardware reset, the
    /// ADC needs to be initialized again, using [`Self::init()`].
    pub fn hard_reset(&mut self) -> Result<(), Error<S, P>> {
        self.adc.hard_reset()
    }

    /// Performs a software reset of the ADC. During a software reset, all the internal registers
    /// are reset to their default values. The dc-to-dc converter continues to function. After a
    /// software reset, the ADC needs to be initialized again, using [`Self::init()`].
    pub fn soft_reset(&mut self) -> Result<(), Error<S, P>> {
        self.adc.soft_reset()
    }

    /// Powers-down the ADC by turning off the dc-to-dc converter and shutting down the Σ-Δ
    /// modulators. Although the ADE7912/ADE7913 configuration registers maintain their values, the
    /// `iwv`, `v1wv`, and `v2wv` [`Measurement`] fields are in an undefined state.
    pub fn powerdown(&mut self) -> Result<(), Error<S, P>> {
        self.adc.powerdown()
    }

    /// Wakes-up the ADC by turning on the dc-to-dc converter and activating the Σ-Δ modulators.
    pub fn wakeup(&mut self) -> Result<(), Error<S, P>> {
        self.adc.wakeup()
    }

    /// Returns the latest available measurement from the ADC as a [`RawMeasurement`] struct. Call
    /// this method inside the ISR from the DREADY pin to get a new measurement as soon as it's
    /// ready. This method does not convert the received data. To get converted metrics, use
    /// [`Self::get_measurement()`] instead. This method does not perform CRC checks on received
    /// data.
    pub fn get_raw_measurement(&mut self) -> Result<RawMeasurement, Error<S, P>> {
        self.adc.get_raw_measurement().map(|m| m[0])
    }

    /// Returns the latest available measurement from the ADC as a [`Measurement`] struct. Call this
    /// method inside the ISR from the DREADY pin to get a new measurement as soon as it's ready.
    /// This method converts raw data to voltage, current and temperature measurements using the
    /// provided calibration values. This method does not perform CRC checks on received data.
    pub fn get_measurement(&mut self) -> Result<Measurement, Error<S, P>> {
        self.adc.get_measurement().map(|m| m[0])
    }
}

/// Contains the raw values coming from the ADC.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RawMeasurement {
    /// Raw current channel value.
    pub iwv: i32,
    /// Raw voltage 1 channel value.
    pub v1wv: i32,
    /// Raw voltage 2 channel value.
    pub v2wv: i32
}

/// Contains the converted metrics coming from the ADC.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Measurement {
    /// Current value in Amperes.
    pub current: f32,
    /// Voltage value in Volts.
    pub voltage: f32,
    /// Auxiliary metric value as a [`MeasurementAux`]. This field can be a second voltage
    /// measurement in Volts for the ADE7913 or a temperature measurement in °C for the ADE7912 or
    /// the ADE7913, if `temp_en = true` in [`Config`].
    pub aux: MeasurementAux
}

/// Represents the possible auxiliary measurement metrics.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MeasurementAux {
    Voltage(f32),
    Temperature(f32)
}

/// Contains the calibration values for the ADC.
#[derive(Default, Debug, Copy, Clone)]
pub struct Calibration {
    /// Calibration offset as a [`CalibrationOffset`].
    pub offset: CalibrationOffset,
    /// Calibration gain as a [`CalibrationGain`].
    pub gain: CalibrationGain
}

/// Contains the calibration offsets, that can be obtained by reading the ADC measurements with the
/// default calibration values and no load applied.
#[derive(Default, Debug, Copy, Clone)]
pub struct CalibrationOffset {
    /// Calibration offset for the current channel.
    pub current: f32,
    /// Calibration offset for the voltage channel.
    pub voltage: f32,
    /// Calibration offset for the auxiliary channel. Set this field to [`None`] to automatically
    /// set the auxiliary offset based on the internal values.
    pub aux: Option<f32>
}

/// Contains the calibration multipliers, that can be obtained by applying a reference load and
/// dividing it with the ADC measurements with only the offset values set, while leaving the
/// multipliers to their default values.
#[derive(Debug, Copy, Clone)]
pub struct CalibrationGain {
    /// Calibration gain for the current channel.
    pub current: f32,
    /// Calibration voltage for the voltage channel.
    pub voltage: f32,
    /// Calibration gain for the auxiliary channel. Set this field to [`None`] to automatically
    /// set the auxiliary offset based on the internal values.
    pub aux: Option<f32>
}

impl Default for CalibrationGain {
    fn default() -> Self {
        Self {
            current: 1.0,
            voltage: 1.0,
            aux: None
        }
    }
}

/// Represents the chips of the ADE791x family.
#[derive(PartialEq, Eq)]
pub enum Chip {
    ADE7912,
    ADE7913
}

/// Represents the possible errors.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Error<S, P> {
    SpiError(S),
    PinError(P),
    ResetTimeout,
    ReadOnlyRegister,
    WriteOnlyRegister,
    BurstReadNotPermitted,
    RegisterContentMismatch
}
