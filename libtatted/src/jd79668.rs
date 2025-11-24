use crate::{InkyError, InkyImage, InkyResult, Resolution};
use camino::Utf8PathBuf;
use gpiocdev::Request;
use gpiocdev::line::{Bias, Direction, Value};
use log::debug;
use serde::{Deserialize, Serialize};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

// TODO: need docs for these
const X_ADDR_START_H: u8 = 0x01;
const X_ADDR_START_L: u8 = 0x90;
const Y_ADDR_START_H: u8 = 0x01;
const Y_ADDR_START_L: u8 = 0x2C;

/// Maximum size of a single SPI transmission frame in bytes.
const SPI_CHUNK_SIZE: usize = 4096;

/// Configuration for GPIOs required for Inky displays.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Jd79668GpiosConfig {
    /// Path to the GPIO chip
    pub gpio_chip: Utf8PathBuf,

    /// Manually manipulated SPI chip select pin
    pub chip_select: u32,

    /// Data/command pin
    pub data_cmd: u32,

    /// Hardware reset pin
    pub reset: u32,

    /// Display busy indicator pin
    pub busy: u32,
}

/// Owned GPIO lines on a specific chip.
pub struct Jd79668Gpios {
    /// Manually manipulated SPI chip select pin
    pub chip_select: Request,

    /// Data/command pin
    pub data_cmd: Request,

    /// Hardware reset pin
    pub reset: Request,

    /// Display busy indicator pin
    pub busy: Request,
}

impl Jd79668Gpios {
    /// Take ownership of the GPIO lines configured in the [`Jd79668GpiosConfig`].
    pub fn from_config(cfg: Jd79668GpiosConfig) -> Result<Self, gpiocdev::Error> {
        // TODO (tff): a gpiocdev::Request::Config might make this nicer
        let chip_select = gpiocdev::Request::builder()
            .on_chip(&cfg.gpio_chip)
            .with_line(cfg.chip_select)
            .with_direction(Direction::Output)
            .with_value(Value::Active)
            .with_bias(Bias::Disabled)
            .with_consumer("tatted-cs")
            .request()?;

        let data_cmd = gpiocdev::Request::builder()
            .on_chip(&cfg.gpio_chip)
            .with_line(cfg.data_cmd)
            .with_direction(Direction::Output)
            .with_value(Value::Inactive)
            .with_bias(Bias::Disabled)
            .with_consumer("tatted-dc")
            .request()?;

        let reset = gpiocdev::Request::builder()
            .on_chip(&cfg.gpio_chip)
            .with_line(cfg.reset)
            .with_direction(Direction::Output)
            .with_value(Value::Active)
            .with_bias(Bias::Disabled)
            .with_consumer("tatted-reset")
            .request()?;

        let busy = gpiocdev::Request::builder()
            .on_chip(&cfg.gpio_chip)
            .with_line(cfg.busy)
            .with_direction(Direction::Input)
            .with_bias(Bias::PullUp)
            .with_consumer("tatted-busy")
            .request()?;

        Ok(Self {
            chip_select,
            data_cmd,
            reset,
            busy,
        })
    }
}

/// SPI control registers, section 6 of the user manual. There are others but this is the required
/// set to operate the display.
pub enum Jd79668Commands {
    PanelSetting = 0x00,
    PowerSetting = 0x01,
    PowerOff = 0x02,
    PowerOn = 0x04,
    BoosterSoftStart = 0x06,
    DeepSleep = 0x07,
    DataStartTransmission = 0x10,
    DataStopTransmission = 0x11, // TODO: unused currently
    DisplayRefresh = 0x12,
    AutoSequence = 0x17, // TODO: unused currently
    VcomDataIntervalSetting = 0x50,
    ResolutionSetting = 0x61,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Jd79668Config {
    pub display_res: Resolution,
    pub spi_path: Utf8PathBuf,
    pub gpios: Jd79668GpiosConfig,
}

impl Default for Jd79668Config {
    fn default() -> Self {
        Self {
            display_res: Resolution::new(400, 300),
            spi_path: Utf8PathBuf::from("/dev/spidev0.0"),

            // RPi pinout for the wHAT available here: https://pinout.xyz/pinout/inky_what
            gpios: Jd79668GpiosConfig {
                gpio_chip: Utf8PathBuf::from("/dev/gpiochip0"),
                chip_select: 8,
                data_cmd: 22,
                reset: 27,
                busy: 17,
            },
        }
    }
}

/// Represents a physical Inky e-ink display with SPI, GPIO, and i2c connections
pub struct InkyJd79668 {
    spi: Spidev,
    gpios: Jd79668Gpios,
    display_res: Resolution,
    initialized: bool,
}

// TODO (tff): would like to incorporate the EEPROM read here as well
impl InkyJd79668 {
    /// Take ownership of all required peripheral hardware and return a new [`InkyJd79668`].
    ///
    /// The display will need to be initialized with [`Self::initialize`] before new images can be
    /// rendered on the display.
    pub fn new(cfg: Jd79668Config) -> InkyResult<Self> {
        debug!("initializing a new inky display, taking GPIOs");

        // Prepare GPIO lines
        let gpios = Jd79668Gpios::from_config(cfg.gpios)?;

        debug!("taking SPI device");

        // Prepare SPI device
        let mut spi = Spidev::open(cfg.spi_path)?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(1_000_000)
            .mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS)
            .build();
        spi.configure(&options)?;

        debug!("successfully took ownership of required peripherals");

        Ok(Self {
            spi,
            gpios,
            display_res: cfg.display_res,
            initialized: false,
        })
    }

    /// Perform a hardware reset
    pub fn hardware_reset(&mut self) -> InkyResult<()> {
        debug!("performing a display hardware reset");

        // The sleeps are stolen from the inky python lib, haven't tested to see how necessary they are
        self.gpios.reset.set_lone_value(Value::Inactive)?;
        thread::sleep(Duration::from_millis(100));

        self.gpios.reset.set_lone_value(Value::Active)?;
        thread::sleep(Duration::from_millis(100));

        Ok(())
    }

    /// Initialize the display
    pub fn initialize(&mut self) -> InkyResult<()> {
        use Jd79668Commands as Cmd;

        debug!("running display initialization");

        self.hardware_reset()?;

        self.busy_wait(Duration::from_secs(1))?;

        // Initialization sequence is ripped directly from the python impl
        self.send_command(0x4D, Some(&[0x78]))?;
        self.send_command(Cmd::PanelSetting as u8, Some(&[0x0F, 0x29]))?;
        self.send_command(
            Cmd::BoosterSoftStart as u8,
            Some(&[0x0d, 0x12, 0x24, 0x25, 0x12, 0x29, 0x10]),
        )?;
        self.send_command(0x30, Some(&[0x08]))?;
        self.send_command(Cmd::VcomDataIntervalSetting as u8, Some(&[0x37]))?;
        self.send_command(
            Cmd::ResolutionSetting as u8,
            Some(&[
                X_ADDR_START_H,
                X_ADDR_START_L,
                Y_ADDR_START_H,
                Y_ADDR_START_L,
            ]),
        )?;

        // TODO: better docs here
        self.send_command(0xae, Some(&[0xcf]))?;
        self.send_command(0xb0, Some(&[0x13]))?;
        self.send_command(0xbd, Some(&[0x07]))?;
        self.send_command(0xbe, Some(&[0xfe]))?;
        self.send_command(0xE9, Some(&[0x01]))?;

        self.initialized = true;

        Ok(())
    }

    /// Send a SPI command
    fn send_command(&mut self, command: u8, data: Option<&[u8]>) -> InkyResult<()> {
        self.gpios.chip_select.set_lone_value(Value::Inactive)?;
        self.gpios.data_cmd.set_lone_value(Value::Inactive)?;

        thread::sleep(Duration::from_millis(300));

        self.spi.write_all(&[command])?;

        // Now send the data if some is provided
        if let Some(data) = data {
            self.gpios.data_cmd.set_lone_value(Value::Active)?;

            thread::sleep(Duration::from_millis(300));

            // Chunk up the write if necessary
            if data.len() <= SPI_CHUNK_SIZE {
                self.spi.write_all(data)?;
            } else {
                for chunk in data.chunks(SPI_CHUNK_SIZE) {
                    self.spi.write_all(chunk)?;
                }
            }
        }

        self.gpios.chip_select.set_lone_value(Value::Active)?;
        self.gpios.data_cmd.set_lone_value(Value::Inactive)?;

        Ok(())
    }

    /// Block waiting on the "busy" GPIO pin to indicate that the display is ready for new data or commands.
    fn busy_wait(&mut self, timeout: Duration) -> InkyResult<()> {
        let start = Instant::now();

        // Note that the inky python impl from Pimoroni has sketchy looking logic in here
        // where if we call busy_wait() and the display is immediately "not busy", we wait for
        // an arbitrary time out anyway. This doesn't seem to be required in practice so I'm leaving
        // it out here.
        while start.elapsed() < timeout {
            match self.gpios.busy.lone_value()? {
                // Display is busy
                Value::Inactive => {
                    // TODO (tff): there's almost certainly a better API in gpiocdev for doing this without a hotloop
                    thread::sleep(Duration::from_millis(10));
                }
                // Display is not busy
                Value::Active => return Ok(()),
            }
        }

        Err(InkyError::BusyTimeout { timeout })
    }

    /// Send a SPI command and then wait for the busy GPIO pin to indicate that the display is ready for new data
    /// or commands.
    fn send_command_wait(
        &mut self,
        command: u8,
        data: Option<&[u8]>,
        timeout: Duration,
    ) -> InkyResult<()> {
        self.send_command(command, data)?;
        self.busy_wait(timeout)?;

        Ok(())
    }

    /// Refresh the display with the [`InkyImage`], this clones the stored palletized image.
    pub fn show(&mut self, img: &InkyImage) -> InkyResult<()> {
        use Jd79668Commands as Cmd;

        debug!("attempting to show a new image on display");

        if !self.initialized {
            return Err(InkyError::Uninitialized);
        }

        let img_res = img.resolution();
        if img_res != self.display_res {
            return Err(InkyError::UnsupportedResolution {
                expected: self.display_res,
                found: img_res,
            });
        }

        // A nice liberal timeout, most commands won't use anywhere near all of this except for the display refresh.
        let timeout = Duration::from_secs(40);

        let packed = Self::pack_buffer(&img.index_img().into_vec())?;
        self.send_command(Cmd::DataStartTransmission as u8, Some(&packed))?;

        // TODO (tff): this whole power on, display refresh, power off, deep sleep
        // sequence is the 0x17 "Auto Sequence". maybe just use that?

        self.send_command_wait(Cmd::PowerOn as u8, None, timeout)?;
        self.send_command_wait(Cmd::DisplayRefresh as u8, Some(&[0x00]), timeout)?;
        self.send_command_wait(Cmd::PowerOff as u8, Some(&[0x00]), timeout)?;
        self.send_command_wait(Cmd::DeepSleep as u8, Some(&[0xa5]), timeout)?;

        Ok(())
    }

    // TODO (tff): unit test this
    //
    /// Pack a palletized image into a flattened 2-bit-per-pixel buffer to be sent via SPI to the display.
    ///
    /// Input: each pixel is a u8 ∈ {0,1,2,3}
    ///
    /// Output: Vec<u8> where each byte contains 4 pixels:
    /// [p0 p1 p2 p3] → (p0<<6) | (p1<<4) | (p2<<2) | (p3)
    ///
    /// Returns an error if any pixel value > 3.
    #[allow(clippy::get_first)]
    fn pack_buffer(pixels: &[u8]) -> InkyResult<Vec<u8>> {
        if !pixels.iter().all(|&p| p < 4) {
            return Err(InkyError::InvalidPalettization {
                index_min: 0,
                index_max: 3,
            });
        }

        let mut out = Vec::with_capacity(pixels.len().div_ceil(4));

        for chunk in pixels.chunks(4) {
            let p0 = *chunk.get(0).unwrap_or(&0);
            let p1 = *chunk.get(1).unwrap_or(&0);
            let p2 = *chunk.get(2).unwrap_or(&0);
            let p3 = *chunk.get(3).unwrap_or(&0);

            let byte = (p0 << 6) | (p1 << 4) | (p2 << 2) | (p3);

            out.push(byte);
        }

        Ok(out)
    }
}
