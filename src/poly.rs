use super::*;

/// Represents multiple ADE7912/ADE7913 3-Channel, Isolated, Sigma-Delta ADCs with SPI configured in
/// a polyphase metering system.
pub struct Ade791x<SPI, CS, const N: usize> {
    spi: SPI,
    adcs: [ade791x::Ade791x<SPI, CS>; N],
}

impl<SPI, CS, S, P, const N: usize> Ade791x<SPI, CS, N>
where
    SPI: spi::Transfer<u8, Error = S>,
    CS: OutputPin<Error = P>,
{
    /// Creates a new [`Ade791x`] instance, given the SPI peripheral and an array of the CS output
    /// pins and chips. The newly created instance must be initialized using [`Self::init()`].
    /// # Arguments
    /// * `spi` - The SPI interface implementing the [`spi::Transfer`] trait.
    /// * `adcs` - The array of tuples containing the CS output pins (implementing the
    /// [`OutputPin`]) trait and the chips as [`Chip`].
    pub fn new(spi: SPI, adcs: [(CS, Chip); N]) -> Self {
        Self {
            spi,
            adcs: adcs.map(|(cs, chip)| ade791x::Ade791x::new(cs, chip)),
        }
    }

    /// Initializes the ADCs, applying the given configurations. After this method, the ADCs are
    /// ready to use.
    /// # Arguments
    /// * `delay` - The delay source implementing the [`DelayMs`] trait.
    /// * `config` - An array of [`Config`] structs containing the configurations for the ADCs.
    /// * `calibration` - An array of [`Calibration`] structs containing the calibration values for
    /// the ADCs.
    /// * `emi_ctrl` - An array of [`EmiCtrl`] structs containing the EMI settings for the ADCs.
    pub fn init(
        &mut self,
        delay: &mut dyn DelayMs<u32>,
        config: [Config; N],
        calibration: [Calibration; N],
        emi_ctrl: [EmiCtrl; N],
    ) -> Result<(), Error<S, P>> {
        for i in 0..N {
            self.adcs[i].init(&mut self.spi, delay, config[i], calibration[i], emi_ctrl[i])?;
        }
        if N > 1 {
            self.sync()?;
        }
        self.lock()?;
        Ok(())
    }

    /// Performs a hardware reset of the ADCs. During a hardware reset, all the registers are set to
    /// their default values and the dc-to-dc converters are shut down. After a hardware reset, the
    /// ADCs need to be initialized again, using [`Self::init()`].
    pub fn hard_reset(&mut self) -> Result<(), Error<S, P>> {
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_listen()?;
        }
        self.adcs[0].hard_reset(&mut self.spi)?;
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_end()?;
        }
        Ok(())
    }

    /// Performs a software reset of the ADCs. During a software reset, all the internal registers
    /// are reset to their default values. The dc-to-dc converters continue to function. After a
    /// software reset, the ADCs need to be initialized again, using [`Self::init()`].
    pub fn soft_reset(&mut self) -> Result<(), Error<S, P>> {
        self.unlock()?;
        for adc in &mut self.adcs {
            adc.soft_reset(&mut self.spi)?;
        }
        Ok(())
    }

    /// Powers-down the ADCs by turning off the dc-to-dc converters and shutting down the Σ-Δ
    /// modulators. Although the ADE7912/ADE7913 configuration registers maintain their values, the
    /// `iwv`, `v1wv`, and `v2wv` [`Measurement`] fields are in an undefined state.
    pub fn powerdown(&mut self) -> Result<(), Error<S, P>> {
        self.unlock()?;
        for adc in &mut self.adcs {
            adc.powerdown(&mut self.spi)?;
        }
        self.lock()?;
        Ok(())
    }

    /// Wakes-up the ADCs by turning on the dc-to-dc converters and activating the Σ-Δ modulators.
    pub fn wakeup(&mut self) -> Result<(), Error<S, P>> {
        self.unlock()?;
        for adc in &mut self.adcs {
            adc.wakeup(&mut self.spi)?;
        }
        self.lock()?;
        Ok(())
    }

    /// Adjusts the synchronization of the ADCs internal counters by following the procedure
    /// described in the ADE7912/ADE7913 Datasheet. The method compares the value of the counter of
    /// the reference ADC (i.e. the one that is generating the DREADY signal) with the values of the
    /// counters of the other ADCs, adjusting the ADCs that are out of sync.
    pub fn adjust_sync(&mut self) -> Result<[i16; N], Error<S, P>> {
        self.unlock()?;
        self.snap()?;
        let ref_adc_index = self
            .adcs
            .iter()
            .position(|adc| adc.is_dr_source())
            .unwrap_or(0);
        let cref = self.adcs[ref_adc_index].get_cnt_snapshot(&mut self.spi)?;
        let mut drift = [0; N];
        for (i, val) in drift.iter_mut().enumerate() {
            if i == ref_adc_index {
                continue;
            }
            *val = self.adcs[i].adjust_sync(&mut self.spi, cref)?;
        }
        self.lock()?;
        Ok(drift)
    }

    /// Returns the latest available measurement from the ADCs as an array of [`RawMeasurement`]
    /// structs. Call this method inside the ISR from the DREADY pin to get a new measurement as
    /// soon as it's ready. This method does not convert the received data. To get converted
    /// metrics, use [`Self::get_measurement()`] instead. This method does not perform CRC checks on
    /// received data.
    pub fn get_raw_measurement(&mut self) -> Result<[RawMeasurement; N], Error<S, P>> {
        let mut raw_measurement = [RawMeasurement {
            iwv: 0,
            v1wv: 0,
            v2wv: 0,
        }; N];
        for (i, val) in raw_measurement.iter_mut().enumerate() {
            *val = self.adcs[i].get_raw_measurement(&mut self.spi)?;
        }
        Ok(raw_measurement)
    }

    /// Returns the latest available measurement from the ADCs as an array of [`Measurement`]
    /// structs. Call this method inside the ISR from the DREADY pin to get a new measurement as
    /// soon as it's ready. This method converts raw data to voltage and current measurements using
    /// the provided calibration values. This method does not perform CRC checks on received data.
    pub fn get_measurement(&mut self) -> Result<[Measurement; N], Error<S, P>> {
        let mut measurement = [Measurement {
            current: 0.0,
            voltage: 0.0,
            aux: MeasurementAux::Voltage(0.0),
        }; N];
        for (i, val) in measurement.iter_mut().enumerate() {
            *val = self.adcs[i].get_measurement(&mut self.spi)?;
        }
        Ok(measurement)
    }

    /// Performs the synchronization procedure for the ADCs. After this procedure, the internal
    /// counters of the ADCs are aligned. This method should be called only during initialization,
    /// as it invalidates the readings for some ADC cycles. For aligning the counters during
    /// operation, use [`Self::adjust_sync()`].
    fn sync(&mut self) -> Result<(), Error<S, P>> {
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_listen()?;
        }
        self.adcs[0].sync(&mut self.spi)?;
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_end()?;
        }
        Ok(())
    }

    /// Performs the snap procedure for the ADCs. After this procedure, the values of the internal
    /// internal counters of the ADCs is captured at the same time and stored in the CNT_SNAPSHOT
    /// register, that can be read using [`Self::get_measurement()`].
    fn snap(&mut self) -> Result<(), Error<S, P>> {
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_listen()?;
        }
        self.adcs[0].snap(&mut self.spi)?;
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_end()?;
        }
        Ok(())
    }

    /// Locks the internal register of the ADCs, meaning that they cannot be written.
    fn lock(&mut self) -> Result<(), Error<S, P>> {
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_listen()?;
        }
        self.adcs[0].lock(&mut self.spi)?;
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_end()?;
        }
        Ok(())
    }

    /// Unlocks the internal register of the ADC, meaning that they can be written.
    fn unlock(&mut self) -> Result<(), Error<S, P>> {
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_listen()?;
        }
        self.adcs[0].unlock(&mut self.spi)?;
        for adc in &mut self.adcs[1.min(N)..] {
            adc.broadcast_end()?;
        }
        Ok(())
    }
}
