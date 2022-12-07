/// Configuration struct.
#[derive(Default, Debug, Copy, Clone)]
pub struct Config {
    /// Enables CLKOUT functionality at the CLKOUT/DREADY pin. When `clkout_en = false`, the default
    /// value, DREADY functionality is enabled. When `clkout_en = true`, CLKOUT functionality is
    /// enabled.
    pub clkout_en: bool,
    /// Shuts down the dc-to-dc converter. When `pwrdwn_en = false`, the default value, the dc-to-dc
    /// converter is functional and the Σ-Δ modulators are active. When `pwrwdn_en = true`, the
    /// dc-to-dc converter is turned off and the Σ-Δ modulators are shut down.
    pub pwrdwn_en: bool,
    /// This field selects the second voltage channel measurement. When the `temp_en` field is set
    /// to `false`, the default value, the voltage between the V2P and VM pins is measured. When
    /// this field is `true`, the internal temperature sensor is measured. In the case of the
    /// ADE7912, the internal temperature sensor is always measured, and this field does not have
    /// any significance.
    pub temp_en: bool,
    /// This field selects the ADC output frequency.
    pub adc_freq: AdcFreqVal,
    /// When this field is set to `true`,a software reset is initiated. This field clears itself to
    /// `false` after one CLKIN cycle.
    pub swrst: bool,
    /// Selects the bandwidth of the digital low-pass filter of the ADC. When `bw = false`, the
    /// default value, the bandwidth is 3.3 kHz. When `bw = true`, the bandwidth is 2 kHz. The
    /// bandwidth data is for CLKIN = 4.096 MHz and an ADC output frequency of 8 kHz.
    pub bw: bool
}

/// Represents the possible ADC frequency values.
#[repr(u8)]
#[derive(Default, Debug, Copy, Clone)]
pub enum AdcFreqVal {
    #[default]
    KHz8 = 0x00,
    KHz4 = 0x01,
    KHz2 = 0x02,
    KHz1 = 0x03
}

impl From<u8> for Config {
    fn from(x: u8) -> Self {
        Self {
            clkout_en: (x & 0x01) != 0,
            pwrdwn_en: (x & 0x04) != 0,
            temp_en: (x & 0x08) != 0,
            adc_freq: AdcFreqVal::from((x & 0x30) >> 4),
            swrst: (x & 0x40) != 0,
            bw: (x & 0x80) != 0
        }
    }
}

impl From<Config> for u8 {
    fn from(x: Config) -> Self {
        (x.bw as u8) << 7 |
            (x.swrst as u8) << 6 |
            (x.adc_freq as u8) << 4 |
            (x.temp_en as u8) << 3 |
            (x.pwrdwn_en as u8) << 2 |
            (x.clkout_en as u8)
    }
}

impl From<u8> for AdcFreqVal {
    fn from(x: u8) -> Self {
        match x & 0x03 {
            0x00 => AdcFreqVal::KHz8,
            0x01 => AdcFreqVal::KHz4,
            0x02 => AdcFreqVal::KHz2,
            _ => AdcFreqVal::KHz1
        }
    }
}

/// Status struct.
#[derive(Default, Debug, Copy, Clone)]
pub struct Status0 {
    /// During reset, the `reset_on` field is set to `true`. When the reset ends and the
    /// ADE7912/ADE7913 are ready to be configured, the `reset_on` field is cleared to `false`.
    pub reset_on: bool,
    /// If the CRC of the configuration registers changes value, `crc_stat` field is set to `false`.
    pub crc_stat: bool,
    /// If the configuration registers are not protected, this field is `false`. After the
    /// configuration registers are protected, this field is set to `true`.
    pub ic_prot: bool
}

impl From<u8> for Status0 {
    fn from(x: u8) -> Self {
        Status0 {
            reset_on: (x & 0x01) != 0,
            crc_stat: (x & 0x02) != 0,
            ic_prot: (x & 0x04) != 0
        }
    }
}

/// EMI control struct. Manages the PWM control block of the isolated dc-to-dc converter to reduce
/// EMI emissions.
#[derive(Debug, Copy, Clone)]
pub struct EmiCtrl {
    /// Controls the PWM control block pulse during Slot 0 of the CLKIN/4 clock.
    pub slot0: bool,
    /// Controls the PWM control block pulse during Slot 1 of the CLKIN/4 clock.
    pub slot1: bool,
    /// Controls the PWM control block pulse during Slot 2 of the CLKIN/4 clock.
    pub slot2: bool,
    /// Controls the PWM control block pulse during Slot 3 of the CLKIN/4 clock.
    pub slot3: bool,
    /// Controls the PWM control block pulse during Slot 4 of the CLKIN/4 clock.
    pub slot4: bool,
    /// Controls the PWM control block pulse during Slot 5 of the CLKIN/4 clock.
    pub slot5: bool,
    /// Controls the PWM control block pulse during Slot 6 of the CLKIN/4 clock.
    pub slot6: bool,
    /// Controls the PWM control block pulse during Slot 7 of the CLKIN/4 clock.
    pub slot7: bool,
}

impl Default for EmiCtrl {
    fn default() -> Self {
        Self::from(0xFF)
    }
}

impl From<u8> for EmiCtrl {
    fn from(x: u8) -> Self {
        Self {
            slot0: (x & 0x01) != 0,
            slot1: (x & (0x01 << 1)) != 0,
            slot2: (x & (0x01 << 2)) != 0,
            slot3: (x & (0x01 << 3)) != 0,
            slot4: (x & (0x01 << 4)) != 0,
            slot5: (x & (0x01 << 5)) != 0,
            slot6: (x & (0x01 << 6)) != 0,
            slot7: (x & (0x01 << 7)) != 0
        }
    }
}

impl From<EmiCtrl> for u8 {
    fn from(x: EmiCtrl) -> Self {
        (x.slot7 as u8) << 7 |
            (x.slot6 as u8) << 6 |
            (x.slot5 as u8) << 5 |
            (x.slot4 as u8) << 4 |
            (x.slot3 as u8) << 3 |
            (x.slot2 as u8) << 2 |
            (x.slot1 as u8) << 1 |
            (x.slot0 as u8)
    }
}

/// Synchronization struct.
pub(crate) struct SyncSnap {
    /// When the `sync` field is set to `true` via a broadcast SPI write operation, the
    /// ADE7912/ADE7913 devices in the system generate ADC outputs in the same exact moment. The
    /// field clears itself back to `false` after one CLKIN cycle.
    pub sync: bool,
    /// When the `snap` field is set to `true` via a broadcast SPI write operation, the internal
    /// counters of the ADE7912/ADE7913 devices in the system are latched. The field clears itself
    /// back to `false` after one CLKIN cycle.
    pub snap: bool
}

impl From<SyncSnap> for u8 {
    fn from(x: SyncSnap) -> Self {
        (x.snap as u8) << 1 | (x.sync as u8)
    }
}

/// Represents the burst read response coming from the ADC.
#[allow(dead_code)]
pub(crate) struct BurstRead {
    /// Instantaneous value of Current I.
    pub iwv: i32,
    /// Instantaneous value of Voltage V1.
    pub v1wv: i32,
    /// Instantaneous value of Voltage V2.
    pub v2wv: i32,
    /// CRC value of `iwv`, `v1wv`, and `v2wv` fields.
    pub adc_crc: u16,
    /// Status struct.
    pub status0: Status0,
    /// Snapshot value of the counter used in synchronization operation.
    pub cnt_snapshot: u16
}

impl From<[u8; 15]> for BurstRead {
    fn from(x: [u8; 15]) -> Self {
        Self {
            iwv: (i32::from_be_bytes(x[0..4].try_into().unwrap()) << 8) >> 8,
            v1wv: (i32::from_be_bytes(x[3..7].try_into().unwrap()) << 8) >> 8,
            v2wv: (i32::from_be_bytes(x[6..10].try_into().unwrap()) << 8) >> 8,
            adc_crc: u16::from_be_bytes(x[10..12].try_into().unwrap()),
            status0: Status0::from(x[12]),
            cnt_snapshot: u16::from_be_bytes(x[13..].try_into().unwrap())
        }
    }
}

/// Represents the registries of the ADC.
#[repr(u8)]
#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub(crate) enum Register {
    Iwv,
    V1wv,
    V2wv,
    AdcCrc,
    CtrlCrc,
    CntSnapshot,
    Config,
    Status0,
    Lock,
    SyncSnap,
    Counter0,
    Counter1,
    EmiCtrl,
    Status1,
    Tempos
}

impl Register {
    /// Returns the address of the register.
    pub fn addr(&self) -> u8 {
        match self {
            Register::Iwv => 0x00,
            Register::V1wv => 0x01,
            Register::V2wv => 0x02,
            Register::AdcCrc => 0x04,
            Register::CtrlCrc => 0x05,
            Register::CntSnapshot => 0x07,
            Register::Config => 0x08,
            Register::Status0 => 0x09,
            Register::Lock => 0x0A,
            Register::SyncSnap => 0x0B,
            Register::Counter0 => 0x0C,
            Register::Counter1 => 0x0D,
            Register::EmiCtrl => 0x0E,
            Register::Status1 => 0x0F,
            Register::Tempos => 0x18
        }
    }

    /// Returns `true` if the register is read-only, `false` otherwise.
    pub fn is_read_only(&self) -> bool {
        *self != Register::Config &&
            *self != Register::Lock &&
            *self != Register::SyncSnap &&
            *self != Register::Counter0 &&
            *self != Register::Counter1 &&
            *self != Register::EmiCtrl
    }

    /// Returns `true` if the register is write-only, `false` otherwise.
    pub fn is_write_only(&self) -> bool {
        *self == Register::Lock || *self == Register::SyncSnap
    }
}

/// Represent the possible SPI operations and their correspondent code.
#[repr(u8)]
pub(crate) enum SpiOp {
    Read = 0x04,
    Write = 0x00
}

/// Represent the possible lock operations and their correspondent codes.
#[repr(u8)]
pub(crate) enum LockOp {
    Enable = 0xCA,
    Disable = 0x9C
}
