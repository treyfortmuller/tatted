use camino::Utf8PathBuf;
use gpiocdev::chip::Chip;
use i2cdev::linux::{LinuxI2CBus, LinuxI2CError};
use spidev::Spidev;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use tabled::{Table, Tabled};
use tabled::{builder::Builder, settings::Style};

/// An association between the filepath to a GPIO character device, and the result of trying to open it.
// type GpioChipResults = HashMap<Utf8PathBuf, gpiocdev::Result<Chip>>;
#[derive(Debug)]
pub struct GpioChipResults(HashMap<Utf8PathBuf, gpiocdev::Result<Chip>>);

/// An association between the filepath to an I2C character device, and the result of trying to open it.
type I2cBusResults = HashMap<Utf8PathBuf, Result<LinuxI2CBus, LinuxI2CError>>;

/// An association between the filepath to a SPI character device, and the result of trying to open it.
type SpiDevResults = HashMap<Utf8PathBuf, std::io::Result<Spidev>>;

pub struct ProbePeripherals {
    gpio: GpioChipResults,
    i2c: I2cBusResults,
    spi: SpiDevResults,
}

impl ProbePeripherals {
    pub fn probe() -> Self {
        Self {
            gpio: Self::probe_gpios(),
            i2c: Self::probe_i2c(),
            spi: Self::probe_spi(),
        }
    }

    fn probe_spi() -> SpiDevResults {
        let dev_paths = list_matching(Utf8PathBuf::from("/dev"), "spidev");

        let mut res = HashMap::new();

        for path in dev_paths {
            let dev_res = Spidev::open(&path);
            res.insert(path, dev_res);
        }

        res
    }

    fn probe_i2c() -> I2cBusResults {
        let bus_paths = list_matching(Utf8PathBuf::from("/dev"), "i2c-");

        let mut res = HashMap::new();

        // TODO (tff): might want LinuxI2cDevice here and not "Bus"
        for path in bus_paths {
            let bus_res = LinuxI2CBus::new(&path);
            res.insert(path, bus_res);
        }

        res
    }

    fn probe_gpios() -> GpioChipResults {
        let chip_paths = list_matching(Utf8PathBuf::from("/dev"), "gpiochip");

        let mut res = HashMap::new();

        for path in chip_paths {
            let chip_res = Chip::from_path(&path);
            res.insert(path, chip_res);
        }

        GpioChipResults(res)
    }
}

/// Return a list of filepaths to devices in the argument directory with the argument filename prefix.
///
/// e.g. list_matching("/dev", "spidev") -> [ /dev/spidev0.0 ]
fn list_matching(dir: Utf8PathBuf, prefix: &str) -> Vec<Utf8PathBuf> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = dir.read_dir_utf8() {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if name.starts_with(prefix) {
                    entries.push(Utf8PathBuf::from(path));
                }
            }
        }
    }

    entries.sort();
    entries
}

impl fmt::Display for ProbePeripherals {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GPIOs:\n{}", self.gpio)
    }
}

impl fmt::Display for GpioChipResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = Builder::new();

        if self.0.is_empty() {
            return write!(f, "No GPIO devices discovered");
        }

        for (path, res) in self.0.iter() {
            let res_str = match res {
                Ok(chip) => {
                    // Have never run into this problem in practice
                    let info = chip.info().expect(
                        "failed to get GPIO chip info after successfully opening the device",
                    );
                    format!(
                        "name: {}, label: {}, num_lines: {}",
                        info.name, info.label, info.num_lines
                    )
                }
                Err(e) => {
                    format!("error: {:#?}", e)
                }
            };

            builder.push_record([path.to_string(), res_str]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        write!(f, "{}", table)
    }
}

// #[derive(Debug, Default)]
// pub struct ProbeInfo {
//     pub eeprom: Option<EepromInfo>,
//     pub eeprom_error: Option<String>,
//     pub display: Option<DisplaySpec>,
//     pub eeprom_bus: Option<PathBuf>,
//     pub spi_devices: Vec<PathBuf>,
//     pub gpio_chips: Vec<PathBuf>,
//     pub gpio_chip_labels: Vec<String>,
//     pub i2c_buses: Vec<PathBuf>,
//     pub i2c_bus_results: Vec<I2cBusReport>,
// }

// pub fn probe_system() -> ProbeInfo {
//     let mut info = ProbeInfo::default();

//     info.spi_devices = list_matching("/dev", "spidev");
//     info.gpio_chips = list_matching("/dev", "gpiochip");
//     info.i2c_buses = list_matching("/dev", "i2c-");
//     info.gpio_chip_labels = list_gpio_chip_labels(&info.gpio_chips);

//     for bus in &info.i2c_buses {
//         let status = read_eeprom(bus);
//         info.i2c_bus_results.push(I2cBusReport {
//             path: bus.clone(),
//             status: status.clone(),
//         });

//         match status {
//             I2cProbeStatus::Found(eeprom) => {
//                 if info.eeprom.is_none() {
//                     info.display = eeprom.display_spec();
//                     info.eeprom = Some(eeprom);
//                     info.eeprom_bus = Some(bus.clone());
//                     info.eeprom_error = None;
//                 }
//             }
//             I2cProbeStatus::Invalid(reason) => {
//                 if info.eeprom_error.is_none() {
//                     info.eeprom_error = Some(format!("invalid data: {reason}"));
//                 }
//             }
//             I2cProbeStatus::Error(err) => {
//                 if info.eeprom_error.is_none() {
//                     info.eeprom_error = Some(err);
//                 }
//             }
//             _ => {}
//         }
//     }

//     info
// }
