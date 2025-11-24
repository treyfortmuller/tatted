use camino::Utf8PathBuf;
use gpiocdev::chip::Chip;
use i2cdev::linux::{LinuxI2CBus, LinuxI2CError};
use spidev::Spidev;
use std::collections::HashMap;
use std::fmt;
use tabled::{builder::Builder, settings::Style};

/// An association between the filepath to a GPIO character device, and the result of trying to open it.
// type GpioChipResults = HashMap<Utf8PathBuf, gpiocdev::Result<Chip>>;
#[derive(Debug)]
pub struct GpioChipResults(HashMap<Utf8PathBuf, gpiocdev::Result<Chip>>);

/// An association between the filepath to an I2C character device, and the result of trying to open it.
#[derive(Debug)]
pub struct I2cBusResults(HashMap<Utf8PathBuf, Result<LinuxI2CBus, LinuxI2CError>>);

/// An association between the filepath to a SPI character device, and the result of trying to open it.
#[derive(Debug)]
pub struct SpiDevResults(HashMap<Utf8PathBuf, std::io::Result<Spidev>>);

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

    fn probe_gpios() -> GpioChipResults {
        let chip_paths = list_matching(Utf8PathBuf::from("/dev"), "gpiochip");

        let mut res = HashMap::new();

        for path in chip_paths {
            let chip_res = Chip::from_path(&path);
            res.insert(path, chip_res);
        }

        GpioChipResults(res)
    }

    fn probe_i2c() -> I2cBusResults {
        let bus_paths = list_matching(Utf8PathBuf::from("/dev"), "i2c-");

        let mut res = HashMap::new();

        for path in bus_paths {
            let bus_res = LinuxI2CBus::new(&path);
            res.insert(path, bus_res);
        }

        I2cBusResults(res)
    }

    fn probe_spi() -> SpiDevResults {
        let dev_paths = list_matching(Utf8PathBuf::from("/dev"), "spidev");

        let mut res = HashMap::new();

        for path in dev_paths {
            let dev_res = Spidev::open(&path);
            res.insert(path, dev_res);
        }

        SpiDevResults(res)
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
        write!(
            f,
            "GPIOs:\n{}\ni2c buses:\n{}\nSPI devices:\n{}",
            self.gpio, self.i2c, self.spi
        )
    }
}

//
// A fancy tabled Display will get implemented for each device's newtype, useful for the tatctl CLI.
//

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

        let table = builder.build();
        write!(f, "{}", table)
    }
}

impl fmt::Display for I2cBusResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = Builder::new();

        if self.0.is_empty() {
            return write!(f, "No I2C buses discovered");
        }

        for (path, res) in self.0.iter() {
            let res_str = match res {
                Ok(_) => {
                    format!("Successfully opened I2C bus")
                }
                Err(e) => {
                    format!("error: {:#?}", e)
                }
            };

            builder.push_record([path.to_string(), res_str]);
        }

        let table = builder.build();
        write!(f, "{}", table)
    }
}

impl fmt::Display for SpiDevResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = Builder::new();

        if self.0.is_empty() {
            return write!(f, "No SPI devices discovered");
        }

        for (path, res) in self.0.iter() {
            let res_str = match res {
                Ok(_) => {
                    format!("Successfully opened SPI device")
                }
                Err(e) => {
                    format!("error: {:#?}", e)
                }
            };

            builder.push_record([path.to_string(), res_str]);
        }

        let table = builder.build();
        write!(f, "{}", table)
    }
}
