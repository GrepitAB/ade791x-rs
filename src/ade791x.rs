use super::*;

/// Internal struct representing a single ADE7912/ADE7913 3-Channel, Isolated, Sigma-Delta ADC with
/// SPI. This struct does not own the SPI interface, making it suitable for polyphase
/// configurations.
pub(crate) struct Ade791x<SPI, CS> {
    _spi: PhantomData<SPI>,
    cs: CS,
    chip: Chip,
    config: Config,
    calibration: Calibration
}

impl<SPI, CS, S, P> Ade791x<SPI, CS>
    where
        SPI: spi::Transfer<u8, Error=S>,
        CS: OutputPin<Error = P> {

    /// Creates a new [`Ade791x`] instance, given the CS output pin. The newly created instance must
    /// be initialized using [`Self::init()`].
    /// # Arguments
    /// * `cs` - The CS output pin implementing the [`OutputPin`] trait.
    /// * `chip` - The chip version as a [`Chip`].
    pub fn new(cs: CS, chip: Chip) -> Self {
        Self { _spi: PhantomData, chip, cs, config: Config::default(), calibration: Calibration::default() }
    }

    /// Initializes the ADC, applying the given configuration. After this method, the ADC is ready
    /// to use.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `delay` - The delay source implementing the [`DelayMs`] trait.
    /// * `config` - The [`Config`] struct containing the configuration for the ADC.
    /// * `calibration` - The [`Calibration`] struct containing the calibration values for the ADC.
    /// * `emi_ctrl` - The [`EmiCtrl`] struct containing the EMI settings for polyphase
    /// configurations.
    pub fn init(&mut self,
                spi: &mut SPI,
                delay: &mut dyn DelayMs<u32>,
                config: Config,
                calibration: Calibration,
                emi_ctrl: EmiCtrl) -> Result<(), Error<S, P>> {
        self.config = config;
        self.calibration = calibration;
        self.wait_reset(spi, delay)?;
        self.write_reg_checked(spi, Register::Config, self.config.into())?;
        self.write_reg_checked(spi, Register::EmiCtrl, emi_ctrl.into())?;
        if self.calibration.offset.aux.is_none() {
            self.calibration.offset.aux = if self.config.temp_en || self.chip == Chip::ADE7912 {
                Some(self.read_reg(spi, Register::Tempos)?[1] as i8 as f32)
            } else { Some(0.0) };
        }
        if self.calibration.gain.aux.is_none() {
            self.calibration.gain.aux = if self.config.temp_en || self.chip == Chip::ADE7912 {
                if self.config.bw { Some(8.21015e-5) }
                else { Some(8.72101e-5) }
            } else { Some(1.0) }
        }
        Ok(())
    }

    /// Returns `true` if the ADC is generating the DREADY signal, `false` if it is generating the
    /// CLKOUT signal instead.
    pub fn is_dr_source(&self) -> bool {
        !self.config.clkout_en
    }

    /// Performs a hardware reset of the ADC. During a hardware reset, all the registers are set to
    /// their default values and the dc-to-dc converter is shut down. After a hardware reset, the
    /// ADC needs to be initialized again, using [`Self::init()`].
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn hard_reset(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        self.cs.set_low().map_err(Error::PinError)?;
        spi.transfer(&mut [0; 8]).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }

    /// Performs a software reset of the ADC. During a software reset, all the internal registers
    /// are reset to their default values. The dc-to-dc converter continues to function. After a
    /// software reset, the ADC needs to be initialized again, using [`Self::init()`].
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn soft_reset(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        let config = Config { swrst: true, ..Default::default() };
        self.write_reg(spi, Register::Config, config.into())
    }

    /// Waits for the reset (either hardware or software) to be completed. The function timeouts
    /// returning a [`Error::ResetError`] after about 500 ms.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `delay` - The delay source implementing the [`DelayMs`] trait.
    pub fn wait_reset(&mut self,
                      spi: &mut SPI,
                      delay: &mut dyn DelayMs<u32>) -> Result<(), Error<S, P>> {
        for _ in 0..5 {
            if !Status0::from(self.read_reg(spi, Register::Status0)?[1]).reset_on {
                return Ok(())
            }
            delay.delay_ms(100);
        }
        Err(Error::ResetTimeout)
    }

    /// Powers-down the ADC by turning off the dc-to-dc converter and shutting down the Σ-Δ
    /// modulators. Although the ADE7912/ADE7913 configuration registers maintain their values, the
    /// `iwv`, `v1wv`, and `v2wv` [`Measurement`] fields are in an undefined state.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn powerdown(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        let mut config = self.config.clone();
        config.pwrdwn_en = true;
        config.clkout_en = false;
        self.write_reg(spi, Register::Config, config.into())
    }

    /// Wakes-up the ADC by turning on the dc-to-dc converter and activating the Σ-Δ modulators.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn wakeup(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        self.write_reg(spi, Register::Config, self.config.into())
    }

    /// Starts listening for a broadcast send on the SPI bus.
    pub fn broadcast_listen(&mut self) -> Result<(), Error<S, P>> {
        self.cs.set_low().map_err(Error::PinError)
    }

    /// Stops listening for a broadcast send on the SPI bus.
    pub fn broadcast_end(&mut self) -> Result<(), Error<S, P>> {
        self.cs.set_high().map_err(Error::PinError)
    }

    /// Sends a sync command on the SPI bus, if other ADCs are in the broadcast listen mode, they
    /// will receive the command as well. This function is used during a synchronization operation.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn sync(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        let sync = SyncSnap { sync: true, snap: false };
        self.write_reg(spi, Register::SyncSnap, sync.into())
    }

    /// Sends a snap command on the SPI bus, if other ADCs are in the broadcast listen mode, they
    /// will receive the command as well. This function is used during a synchronization operation.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn snap(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        let snap = SyncSnap { sync: false, snap: true };
        self.write_reg(spi, Register::SyncSnap, snap.into())
    }

    /// Adjusts the synchronization of the ADC given the `c0` constant and `cref`, that is the value
    /// of the counter of the reference ADC (i.e. the one that is generating the DREADY signal),
    /// captured with a snap command. The adjustment of the synchronization is done by setting the
    /// starting value of the internal synchronization counter according to the calculated drift
    /// between the internal counter and the one of the reference ADC.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `cref` - The value of the counter of the reference ADC.
    pub fn adjust_sync(&mut self, spi: &mut SPI, cref: u16) -> Result<i16, Error<S, P>> {
        let c0 = match self.config.adc_freq {
            AdcFreqVal::KHz8 => 511,
            AdcFreqVal::KHz4 => 1023,
            AdcFreqVal::KHz2 => 2047,
            AdcFreqVal::KHz1 => 4095
        };
        let c = self.get_cnt_snapshot(spi)?;
        let drift = c as i16 - cref as i16;
        if drift > 1 || drift < -1 {
            let adj = if c > cref { cref + c0 - c } else { cref - c };
            let bytes = adj.to_be_bytes();
            self.write_reg(spi, Register::Counter0, bytes[1])?;
            self.write_reg(spi, Register::Counter1, bytes[0])?;
        }
        Ok(drift)
    }

    /// Locks the internal register of the ADC, meaning that they cannot be written.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn lock(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        self.write_reg(spi, Register::Lock, LockOp::Enable as u8)
    }

    /// Unlocks the internal register of the ADC, meaning that they can be written.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn unlock(&mut self, spi: &mut SPI) -> Result<(), Error<S, P>> {
        self.write_reg(spi, Register::Lock, LockOp::Disable as u8)
    }

    /// Returns the value of the snapshot of the internal counter, triggered with [`Self::snap()`].
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn get_cnt_snapshot(&mut self, spi: &mut SPI) -> Result<u16, Error<S, P>> {
        Ok(BurstRead::from(self.burst_read(spi, Register::Iwv, 9)?).cnt_snapshot)
    }

    /// Returns the latest available measurement from the ADC, without applying any conversion, as
    /// a [`RawMeasurement`] struct.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn get_raw_measurement(&mut self, spi: &mut SPI) -> Result<RawMeasurement, Error<S, P>> {
        let burst_read = BurstRead::from(self.burst_read(spi, Register::Iwv, 9)?);
        Ok(RawMeasurement {
            iwv: burst_read.iwv,
            v1wv: burst_read.v1wv,
            v2wv: burst_read.v2wv
        })
    }

    /// Returns the latest available measurement from the ADC as a [`Measurement`] struct.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    pub fn get_measurement(&mut self, spi: &mut SPI) -> Result<Measurement, Error<S, P>> {
        let raw_measurement = self.get_raw_measurement(spi)?;
        let aux_offset = self.calibration.offset.aux.unwrap_or(0.0);
        let aux_gain = self.calibration.gain.aux.unwrap_or(1.0);
        let mut measurement = Measurement {
            current: Self::map_adc(raw_measurement.iwv, -49.27, 49.27),
            voltage: Self::map_adc(raw_measurement.v1wv, -788.0, 788.0),
            aux: if self.chip == Chip::ADE7912 || self.config.temp_en {
                MeasurementAux::Temperature(aux_gain * raw_measurement.v2wv as f32 + 8.72101e-5 * aux_offset * 2048.0 - 306.47)
            } else {
                MeasurementAux::Voltage((Self::map_adc(raw_measurement.v2wv, -788.0, 788.0) - aux_offset) * aux_gain)
            }
        };
        measurement.current = (measurement.current - self.calibration.offset.current) * self.calibration.gain.current;
        measurement.voltage = (measurement.voltage - self.calibration.offset.voltage) * self.calibration.gain.voltage;
        Ok(measurement)
    }

    /// Returns the given ADC value mapped between the provided output values.
    /// # Arguments
    /// * `x`: The ADC value to be mapped.
    /// * `out_min`: The minimum value of the mapping output.
    /// * `out_max`: The maximum value of the mapping output.
    fn map_adc(x: i32, out_min: f32, out_max: f32) -> f32 {
        (x + 8_388_608) as f32 * (out_max - out_min) / 16_777_215.0 + out_min
    }

    /// Performs a burst read on the SPI bus. This operation is used to get multiple register values
    /// with a single transaction, as for measurement readings. This function returns the whole
    /// transaction buffer with the bytes set according to the `start_reg` and `len` arguments. For
    /// example, if we set `start_reg = Register::Iwv` and `len = 11`, we will receive the
    /// whole transaction buffer with only the first 12 bytes (1 command byte + 11 response bytes)
    /// set, leaving the others empty.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `start_reg` - The starting register as a [`Register`] value.
    /// * `len` - The length of the transaction in terms of number of bytes received.
    fn burst_read(&mut self, spi: &mut SPI, start_reg: Register, len: usize) -> Result<[u8; 15], Error<S, P>> {
        let start_index = match start_reg {
            Register::Iwv => 1,
            Register::V1wv => 4,
            Register::V2wv => 7,
            Register::AdcCrc => 10,
            Register::Status0 => 12,
            Register::CntSnapshot => 13,
            _ => return Err(Error::BurstReadNotPermitted)
        };
        let mut bytes = [0; 15];
        bytes[0] = (start_reg.addr() << 3) | SpiOp::Read as u8;
        self.cs.set_low().map_err(Error::PinError)?;
        spi.transfer(&mut bytes[..len+1]).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        bytes.copy_within(1..len+1, start_index);
        bytes[1..start_index].fill(0);
        Ok(bytes)
    }

    /// Performs a register reading. This method is used to get single byte values from
    /// configuration registers. This function returns the whole transaction buffer, with the first
    /// byte representing the command byte and the second one representing the response byte.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `reg` - The register as a [`Register`] value.
    fn read_reg(&mut self, spi: &mut SPI, reg: Register) -> Result<[u8; 2], Error<S, P>> {
        if reg.is_write_only() { return Err(Error::WriteOnlyRegister) }
        let mut bytes = [(reg.addr() << 3) | SpiOp::Read as u8, 0];
        self.cs.set_low().map_err(Error::PinError)?;
        spi.transfer(&mut bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(bytes)
    }

    /// Performs a register writing. This method is used to set single byte values for configuration
    /// registers.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `reg` - The register as a [`Register`] value.
    /// * `content` - The content to write to the register as an `u8` value.
    fn write_reg(&mut self, spi: &mut SPI, reg: Register, content: u8) -> Result<(), Error<S, P>> {
        if reg.is_read_only() { return Err(Error::ReadOnlyRegister) }
        let mut bytes = [(reg.addr() << 3) | SpiOp::Write as u8, content];
        self.cs.set_low().map_err(Error::PinError)?;
        spi.transfer(&mut bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }

    /// Performs a checked register writing, that means that the written register is read to check
    /// that the data has been actually written. This method is used to set single byte values for
    /// configuration registers.
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `reg` - The register as a [`Register`] value.
    /// * `content` - The content to write to the register as an `u8` value.
    fn write_reg_checked(&mut self, spi: &mut SPI, reg: Register, content: u8) -> Result<(), Error<S, P>> {
        self.write_reg(spi, reg, content)?;
        if self.read_reg(spi, reg)?[1] != content {
            return Err(Error::RegisterContentMismatch);
        }
        Ok(())
    }
}
