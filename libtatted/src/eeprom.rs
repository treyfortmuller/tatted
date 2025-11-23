use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gpiocdev::chip::Chip;
use i2cdev::core::I2CDevice;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

const EEPROM_ADDRESS: u16 = 0x50;
const EEPROM_LENGTH: usize = 29;

const DISPLAY_VARIANT_NAMES: [&str; 25] = [
    "Unknown",
    "Red pHAT (High-Temp)",
    "Yellow wHAT",
    "Black wHAT",
    "Black pHAT",
    "Yellow pHAT",
    "Red wHAT",
    "Red wHAT (High-Temp)",
    "Red wHAT",
    "Unknown",
    "Black pHAT (SSD1608)",
    "Red pHAT (SSD1608)",
    "Yellow pHAT (SSD1608)",
    "Unknown",
    "7-Colour (UC8159) 600x448",
    "7-Colour 640x400 (UC8159)",
    "7-Colour 640x400 (UC8159)",
    "Black wHAT (SSD1683)",
    "Red wHAT (SSD1683)",
    "Yellow wHAT (SSD1683)",
    "7-Colour 800x480 (AC073TC1A)",
    "Spectra 6 13.3 1600x1200 (EL133UF1)",
    "Spectra 6 7.3 800x480 (E673)",
    "Red/Yellow pHAT (JD79661)",
    "Red/Yellow wHAT (JD79668)",
];

#[derive(Clone, Copy, Debug)]
pub struct EepromInfo {
    pub width: u16,
    pub height: u16,
    pub color: u8,
    pub pcb_variant: u8,
    pub display_variant: u8,
}

impl fmt::Display for EepromInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}x{} colour={} pcb_variant={:.1} display_variant={} ({})",
            self.width,
            self.height,
            self.color,
            self.pcb_variant as f32 / 10.0,
            self.display_variant,
            self.variant_name()
        )
    }
}

impl EepromInfo {
    pub fn variant_name(&self) -> &'static str {
        DISPLAY_VARIANT_NAMES
            .get(self.display_variant as usize)
            .copied()
            .unwrap_or("Unknown")
    }

    pub fn display_spec(&self) -> Option<DisplaySpec> {
        match self.display_variant {
            // 14 => Some(DisplaySpec::Uc8159 {
            //     width: 600,
            //     height: 448,
            //     variant: self.display_variant,
            // }),
            // 16 => Some(DisplaySpec::Uc8159 {
            //     width: 640,
            //     height: 400,
            //     variant: self.display_variant,
            // }),
            // 21 => Some(DisplaySpec::El133Uf1 {
            //     width: self.width,
            //     height: self.height,
            // }),
            24 => Some(DisplaySpec::Jd79668 {
                width: self.width,
                height: self.height,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DisplaySpec {
    Jd79668 { width: u16, height: u16 },
}

impl fmt::Display for DisplaySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplaySpec::Jd79668 { width, height } => write!(f, "JD79668 ({}x{})", width, height),
        }
    }
}

#[derive(Clone, Debug)]
pub struct I2cBusReport {
    pub path: PathBuf,
    pub status: I2cProbeStatus,
}

#[derive(Clone, Debug)]
pub enum I2cProbeStatus {
    Found(EepromInfo),
    Blank,
    Invalid(String),
    Unavailable,
    Error(String),
}

pub fn read_eeprom<P: AsRef<Path>>(path: P) -> I2cProbeStatus {
    let path_ref = path.as_ref();
    let mut device = match LinuxI2CDevice::new(path_ref, EEPROM_ADDRESS) {
        Ok(dev) => dev,
        Err(err) => return handle_i2c_open_error(err),
    };

    if let Err(err) = device.write(&[0x00, 0x00]) {
        return map_i2c_error(err);
    }

    let mut buf = [0u8; EEPROM_LENGTH];
    if let Err(err) = device.read(&mut buf) {
        return map_i2c_error(err);
    }

    if is_blank_eeprom(&buf) {
        return I2cProbeStatus::Blank;
    }

    match parse_eeprom(&buf) {
        Ok(parsed) => I2cProbeStatus::Found(parsed),
        Err(reason) => I2cProbeStatus::Invalid(reason),
    }
}

fn parse_eeprom(data: &[u8]) -> Result<EepromInfo, String> {
    let width = u16::from_le_bytes([data[0], data[1]]);
    let height = u16::from_le_bytes([data[2], data[3]]);
    let color = data[4];
    let pcb_variant = data[5];
    let display_variant = data[6];

    if width == 0 || height == 0 || width == u16::MAX || height == u16::MAX {
        return Err(format!(
            "width/height out of range (width={width}, height={height})"
        ));
    }

    if display_variant == u8::MAX {
        return Err("display variant invalid (255)".to_string());
    }

    Ok(EepromInfo {
        width,
        height,
        color,
        pcb_variant,
        display_variant,
    })
}

fn map_i2c_error(err: LinuxI2CError) -> I2cProbeStatus {
    match err {
        LinuxI2CError::Io(io_err) => handle_io_error(io_err),
        LinuxI2CError::Errno(code) => handle_errno(code),
    }
}

fn handle_i2c_open_error(err: LinuxI2CError) -> I2cProbeStatus {
    match err {
        LinuxI2CError::Io(io_err) => handle_io_error(io_err),
        LinuxI2CError::Errno(code) => handle_errno(code),
    }
}

fn handle_io_error(io_err: io::Error) -> I2cProbeStatus {
    match io_err.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => I2cProbeStatus::Unavailable,
        _ => I2cProbeStatus::Error(io_err.to_string()),
    }
}

fn handle_errno(code: i32) -> I2cProbeStatus {
    let io_err = io::Error::from_raw_os_error(code);
    match io_err.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => I2cProbeStatus::Unavailable,
        _ => I2cProbeStatus::Error(io_err.to_string()),
    }
}

fn is_blank_eeprom(data: &[u8]) -> bool {
    data.iter().all(|&b| b == 0xFF || b == 0x00)
}
