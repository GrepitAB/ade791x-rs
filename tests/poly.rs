use ade791x::*;
use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
use embedded_hal_mock::pin::{Mock as PinMock, Transaction as PinTransaction, State as PinState};
use embedded_hal_mock::delay::MockNoop;

#[test]
fn init() {
    let spi_expectations = [
        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x00]),
        // Write/Read CONFIG (checked write)
        SpiTransaction::transfer(vec![0x40, 0x01], vec![0x40, 0x01]),
        SpiTransaction::transfer(vec![0x44, 0x00], vec![0x44, 0x01]),
        // Write/Read EMI_CTRL (checked write)
        SpiTransaction::transfer(vec![0x70, 0x55], vec![0x70, 0x55]),
        SpiTransaction::transfer(vec![0x74, 0x00], vec![0x74, 0x55]),
        // Read TEMPOS (temperature offset)
        SpiTransaction::transfer(vec![0xC4, 0x00], vec![0xC4, 0x5E]),

        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x00]),
        // Write/Read CONFIG (checked write)
        SpiTransaction::transfer(vec![0x40, 0x01], vec![0x40, 0x01]),
        SpiTransaction::transfer(vec![0x44, 0x00], vec![0x44, 0x01]),
        // Write/Read EMI_CTRL (checked write)
        SpiTransaction::transfer(vec![0x70, 0xAA], vec![0x70, 0xAA]),
        SpiTransaction::transfer(vec![0x74, 0x00], vec![0x74, 0xAA]),

        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x00]),
        // Write/Read CONFIG (checked write)
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        SpiTransaction::transfer(vec![0x44, 0x00], vec![0x44, 0x00]),
        // Write/Read EMI_CTRL (checked write)
        SpiTransaction::transfer(vec![0x70, 0x55], vec![0x70, 0x55]),
        SpiTransaction::transfer(vec![0x74, 0x00], vec![0x74, 0x55]),
        // Read TEMPOS (temperature offset)
        SpiTransaction::transfer(vec![0xC4, 0x00], vec![0xC4, 0x6A]),

        // Write SYNC (sync trigger)
        SpiTransaction::transfer(vec![0x58, 0x01], vec![0x58, 0x01]),
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA])
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations[2..]);
    let cs2 = PinMock::new(&cs_expectations);
    let mut delay = MockNoop::new();
    let config = [
        Config { clkout_en: true, ..Default::default() },
        Config { clkout_en: true, ..Default::default() },
        Config::default()
    ];
    let calibration = [Calibration::default(); 3];
    let emi_ctrl = [
        EmiCtrl::from(0x55),
        EmiCtrl::from(0xAA),
        EmiCtrl::from(0x55)
    ];
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7913), (cs2, Chip::ADE7912)
    ]);
    adc.init(&mut delay, config, calibration, emi_ctrl).unwrap();
}

#[test]
fn init_timeout() {
    let spi_expectations = [
        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x01]),
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x01]),
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x01]),
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x01]),
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x01])
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&[]);
    let cs2 = PinMock::new(&[]);
    let mut delay = MockNoop::new();
    let config = [Config::default(); 3];
    let calibration = [Calibration::default(); 3];
    let emi_ctrl = [EmiCtrl::default(); 3];
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7912), (cs2, Chip::ADE7912)
    ]);
    assert_eq!(adc.init(&mut delay, config, calibration, emi_ctrl), Err(Error::ResetTimeout));
}

#[test]
fn hard_reset() {
    let spi_expectations = [
        // Write hard reset sequence
        SpiTransaction::transfer(vec![0x00; 8], vec![0x00; 8])
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations);
    let cs2 = PinMock::new(&cs_expectations);
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7912), (cs2, Chip::ADE7912)
    ]);
    adc.hard_reset().unwrap();
}

#[test]
fn soft_reset() {
    let spi_expectations = [
        // Write LOCK (lock disable)
        SpiTransaction::transfer(vec![0x50, 0x9C], vec![0x50, 0x9C]),
        // Write CONFIG (software reset)
        SpiTransaction::transfer(vec![0x40, 0x40], vec![0x40, 0x40]),
        SpiTransaction::transfer(vec![0x40, 0x40], vec![0x40, 0x40]),
        SpiTransaction::transfer(vec![0x40, 0x40], vec![0x40, 0x40])
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations);
    let cs2 = PinMock::new(&cs_expectations);
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7912), (cs2, Chip::ADE7912)
    ]);
    adc.soft_reset().unwrap();
}

#[test]
fn powerdown() {
    let spi_expectations = [
        // Write LOCK (lock disable)
        SpiTransaction::transfer(vec![0x50, 0x9C], vec![0x50, 0x9C]),
        // Write CONFIG (powerdown enable)
        SpiTransaction::transfer(vec![0x40, 0x04], vec![0x40, 0x04]),
        SpiTransaction::transfer(vec![0x40, 0x04], vec![0x40, 0x04]),
        SpiTransaction::transfer(vec![0x40, 0x04], vec![0x40, 0x04]),
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations);
    let cs2 = PinMock::new(&cs_expectations);
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7912), (cs2, Chip::ADE7912)
    ]);
    adc.powerdown().unwrap();
}

#[test]
fn wakeup() {
    let spi_expectations = [
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0x9C], vec![0x50, 0x9C]),
        // Write CONFIG (powerdown disable)
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        // Write LOCK (lock disable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations);
    let cs2 = PinMock::new(&cs_expectations);
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7912), (cs2, Chip::ADE7912)
    ]);
    adc.wakeup().unwrap();
}

#[test]
fn get_raw_measurement() {
    let spi_expectations = [
        // Burst Read (from IWV to V2WV)
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xEC, 0xDF, 0x06, 0x17, 0x1C, 0x37, 0xBE, 0x97]),
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x06, 0x13, 0x83, 0x05, 0xEC, 0x10, 0x37, 0x9B, 0x6E]),
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xE9, 0x51, 0x06, 0x1A, 0x97, 0x39, 0x6B, 0x84]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations);
    let cs2 = PinMock::new(&cs_expectations);
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7912), (cs2, Chip::ADE7912)
    ]);
    assert_eq!(
        adc.get_raw_measurement().unwrap(),
        [
            RawMeasurement { iwv: 388319, v1wv: 399132, v2wv: 3653271 },
            RawMeasurement { iwv: 398211, v1wv: 388112, v2wv: 3644270 },
            RawMeasurement { iwv: 387409, v1wv: 400023, v2wv: 3763076 }
        ]
    );
}

#[test]
fn get_measurement() {
    let spi_expectations = [
        // Burst Read (from IWV to V2WV)
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xEC, 0xDF, 0x06, 0x17, 0x1C, 0x37, 0xBE, 0x97]),
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x06, 0x13, 0x83, 0x05, 0xEC, 0x10, 0x37, 0x9B, 0x6E]),
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xE9, 0x51, 0x06, 0x1A, 0x97, 0x39, 0x6B, 0x84]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low), PinTransaction::set(PinState::High)
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs0 = PinMock::new(&cs_expectations);
    let cs1 = PinMock::new(&cs_expectations);
    let cs2 = PinMock::new(&cs_expectations);
    let mut adc = poly::Ade791x::new(spi, [
        (cs0, Chip::ADE7912), (cs1, Chip::ADE7913), (cs2, Chip::ADE7912)
    ]);
    assert_eq!(
        adc.get_measurement().unwrap(),
        [
            Measurement {
                current: 2.2807732,
                voltage: 37.493286,
                aux: MeasurementAux::Temperature(3652964.53)
            },
            Measurement {
                current: 2.3388748,
                voltage: 36.45813,
                aux: MeasurementAux::Voltage(342.33167)
            },
            Measurement {
                current: 2.2754288,
                voltage: 37.576965,
                aux: MeasurementAux::Temperature(3762769.53)
            }
        ]
    );
}
