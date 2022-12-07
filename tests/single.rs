use ade791x::*;
use embedded_hal_mock::delay::MockNoop;
use embedded_hal_mock::pin::{Mock as PinMock, State as PinState, Transaction as PinTransaction};
use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};

#[test]
fn init_ade7912() {
    let spi_expectations = [
        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x00]),
        // Write/Read CONFIG (checked write)
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        SpiTransaction::transfer(vec![0x44, 0x00], vec![0x44, 0x00]),
        // Write/Read EMI_CTRL (checked write)
        SpiTransaction::transfer(vec![0x70, 0xFF], vec![0x70, 0xFF]),
        SpiTransaction::transfer(vec![0x74, 0x00], vec![0x74, 0xFF]),
        // Read TEMPOS (temperature offset)
        SpiTransaction::transfer(vec![0xC4, 0x00], vec![0xC4, 0x5E]),
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut delay = MockNoop::new();
    let config = Config::default();
    let calibration = Calibration::default();
    let mut adc = Ade791x::new_ade7912(spi, cs);
    adc.init(&mut delay, config, calibration).unwrap();
}

#[test]
fn init_ade7913() {
    let spi_expectations = [
        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4C, 0x00], vec![0x4C, 0x00]),
        // Write/Read CONFIG (checked write)
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        SpiTransaction::transfer(vec![0x44, 0x00], vec![0x44, 0x00]),
        // Write/Read EMI_CTRL (checked write)
        SpiTransaction::transfer(vec![0x70, 0xFF], vec![0x70, 0xFF]),
        SpiTransaction::transfer(vec![0x74, 0x00], vec![0x74, 0xFF]),
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut delay = MockNoop::new();
    let config = Config::default();
    let calibration = Calibration::default();
    let mut adc = Ade791x::new_ade7913(spi, cs);
    adc.init(&mut delay, config, calibration).unwrap();
}

#[test]
fn init_timeout() {
    let spi_expectations = [
        // Read STATUS0 (wait reset)
        SpiTransaction::transfer(vec![0x4c, 0x00], vec![0x4c, 0x01]),
        SpiTransaction::transfer(vec![0x4c, 0x00], vec![0x4c, 0x01]),
        SpiTransaction::transfer(vec![0x4c, 0x00], vec![0x4c, 0x01]),
        SpiTransaction::transfer(vec![0x4c, 0x00], vec![0x4c, 0x01]),
        SpiTransaction::transfer(vec![0x4c, 0x00], vec![0x4c, 0x01]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut delay = MockNoop::new();
    let config = Config::default();
    let calibration = Calibration::default();
    let mut adc = Ade791x::new_ade7912(spi, cs);
    assert_eq!(
        adc.init(&mut delay, config, calibration),
        Err(Error::ResetTimeout)
    );
}

#[test]
fn hard_reset() {
    let spi_expectations = [
        // Write hard reset sequence
        SpiTransaction::transfer(vec![0x00; 8], vec![0x00; 8]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7912(spi, cs);
    adc.hard_reset().unwrap();
}

#[test]
fn soft_reset() {
    let spi_expectations = [
        // Write LOCK (lock disable)
        SpiTransaction::transfer(vec![0x50, 0x9C], vec![0x50, 0x9C]),
        // Write CONFIG (software reset)
        SpiTransaction::transfer(vec![0x40, 0x40], vec![0x40, 0x40]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7912(spi, cs);
    adc.soft_reset().unwrap();
}

#[test]
fn powerdown() {
    let spi_expectations = [
        // Write LOCK (lock disable)
        SpiTransaction::transfer(vec![0x50, 0x9C], vec![0x50, 0x9C]),
        // Write CONFIG (powerdown enable)
        SpiTransaction::transfer(vec![0x40, 0x04], vec![0x40, 0x04]),
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7912(spi, cs);
    adc.powerdown().unwrap();
}

#[test]
fn wakeup() {
    let spi_expectations = [
        // Write LOCK (lock enable)
        SpiTransaction::transfer(vec![0x50, 0x9C], vec![0x50, 0x9C]),
        // Write CONFIG (powerdown disable)
        SpiTransaction::transfer(vec![0x40, 0x00], vec![0x40, 0x00]),
        // Write LOCK (lock disable)
        SpiTransaction::transfer(vec![0x50, 0xCA], vec![0x50, 0xCA]),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7912(spi, cs);
    adc.wakeup().unwrap();
}

#[test]
fn get_raw_measurement() {
    let spi_expectations = [
        // Burst Read (from IWV to V2WV)
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xEC, 0xDF, 0x06, 0x17, 0x1C, 0x37, 0xBE, 0x97],
        ),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7912(spi, cs);
    assert_eq!(
        adc.get_raw_measurement().unwrap(),
        RawMeasurement {
            iwv: 388319,
            v1wv: 399132,
            v2wv: 3653271
        }
    );
}

#[test]
fn get_measurement_ade7912() {
    let spi_expectations = [
        // Burst Read (from IWV to V2WV)
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xEC, 0xDF, 0x06, 0x17, 0x1C, 0x37, 0xBE, 0x97],
        ),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7912(spi, cs);
    assert_eq!(
        adc.get_measurement().unwrap(),
        Measurement {
            current: 2.2807732,
            voltage: 37.493286,
            aux: MeasurementAux::Temperature(3652964.5)
        }
    );
}

#[test]
fn get_measurement_ade7913() {
    let spi_expectations = [
        // Burst Read (from IWV to V2WV)
        SpiTransaction::transfer(
            vec![0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x04, 0x05, 0xEC, 0xDF, 0x06, 0x17, 0x1C, 0x37, 0xBE, 0x97],
        ),
    ];
    let cs_expectations = [
        PinTransaction::set(PinState::Low),
        PinTransaction::set(PinState::High),
    ];
    let spi = SpiMock::new(&spi_expectations);
    let cs = PinMock::new(&cs_expectations);
    let mut adc = Ade791x::new_ade7913(spi, cs);
    assert_eq!(
        adc.get_measurement().unwrap(),
        Measurement {
            current: 2.2807732,
            voltage: 37.493286,
            aux: MeasurementAux::Voltage(343.17712)
        }
    );
}
