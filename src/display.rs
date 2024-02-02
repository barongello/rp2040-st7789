use core::ops::BitOr;
use cortex_m::{
  delay::Delay,
  prelude::_embedded_hal_blocking_spi_Write
};
use embedded_hal::{
  digital::v2::OutputPin,
  spi::Mode
};
use crate::font::FONT;
use fugit::HertzU32;
use heapless::String;
use rp2040_hal::{
  Spi,
  gpio::{
    FunctionSpi,
    Pin,
    PinId,
    PushPullOutput,
    bank0::{
      Gpio8,
      Gpio9,
      Gpio10,
      Gpio11,
      Gpio12,
      Gpio13
    }
  },
  pac::{
    RESETS,
    SPI1,
  },
  spi::Enabled
};

// const BUFFER_SIZE: u16 = 512;

// https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf
// https://www.rhydolabz.com/documents/33/ST7789.pdf
#[repr(u8)]
#[allow(dead_code)]
pub enum DisplayCommand {
  NOP        = 0x00, // Do nothing
  SWRESET    = 0x01, // Software reset
  RDDID      = 0x04, // Read display ID
  RDDST      = 0x09, // Read display status
  RDDPM      = 0x0A, // Read display power mode
  RDDMADCTL  = 0x0B, // Read display MADCTL
  RDDCOLMOD  = 0x0C, // Read display pixel format
  RDDIM      = 0x0D, // Read display image format
  RDDSM      = 0x0E, // Read display signal mode
  RDDSDR     = 0x0F, // Read display self-diagnostic result
  SLPIN      = 0x10, // Sleep in
  SLPOUT     = 0x11, // Sleep out
  PTLON      = 0x12, // Partial display mode on
  NORON      = 0x13, // Normal display mode on
  INVOFF     = 0x20, // Display inversion off
  INVON      = 0x21, // Display inversion on
  GAMSET     = 0x26, // Gamma set
  DISPOFF    = 0x28, // Display off
  DISPON     = 0x29, // Display on
  CASET      = 0x2A, // Column address set
  RASET      = 0x2B, // Row address set
  RAMWR      = 0x2C, // Memory write
  RAMRD      = 0x2E, // Memory read
  PTLAR      = 0x30, // Partial area
  VSCRDEF    = 0x33, // Vertical scrolling definition
  TEOFF      = 0x34, // Tearing effect line off
  TEON       = 0x35, // Tearing effect line on
  MADCTL     = 0x36, // Memory data access control
  VSCSAD     = 0x37, // Vertical scroll start address of RAM
  IDMOFF     = 0x38, // Idle mode off
  IDMON      = 0x39, // Idle mode on
  COLMOD     = 0x3A, // Interface pixel format
  WRMEMC     = 0x3C, // Write memory continue
  RDMEMC     = 0x3E, // Read memory continue
  STE        = 0x44, // Set tear scanline
  GSCAN      = 0x45, // Get scanline
  WRDISBV    = 0x51, // Write display brightness
  RDDISBV    = 0x52, // Read display brightness value
  WRCTRLD    = 0x53, // Write CTRL display
  RDCTRLD    = 0x54, // Read CTRL value display
  WRCACE     = 0x55, // Write content adaptive brightness control and color enhancement
  RDCABC     = 0x56, // Read content adaptive brightness control
  WRCABCMB   = 0x5E, // Write CACB minimum brightness
  RDCABCMB   = 0x5F, // Read CACB minimum brightness
  RDABCSDR   = 0x68, // Read automatic brightness control self-diagnostic result
  RAMCTRL    = 0xB0, // RAM control
  RGBCTRL    = 0xB1, // RGB interface control
  PORCTRL    = 0xB2, // Porch setting
  FRCTRL1    = 0xB3, // Frame rate control 1 (in partial mode/idle colors)
  PARCTRL    = 0xB5, // Partial control
  GCTRL      = 0xB7, // Gate control
  GTADJ      = 0xB8, // Gate on timing adjustment
  DGMEN      = 0xBA, // Digital gamma enable
  VCOMS      = 0xBB, // VCOM setting
  POWSAVE    = 0xBC, // Power saving mode
  DLPOFFSAVE = 0xBD, // Display off power save
  LCMCTRL    = 0xC0, // LCM control
  IDSET      = 0xC1, // ID code setting
  VDVVRHEN   = 0xC2, // VDV and VRH command enable
  VRHS       = 0xC3, // VRH set
  VDVS       = 0xC4, // VDV set
  VCMOFSET   = 0xC5, // VCOM offset set
  FRCTRL2    = 0xC6, // Frame rate control in normal mode
  CABCCTRL   = 0xC7, // CABC control
  REGSEL1    = 0xC8, // Register value selection 1
  REGSEL2    = 0xCA, // Register value selection 2
  PWMFRSEL   = 0xCC, // PWM frequency selection
  PWCTRL1    = 0xD0, // Power control 1
  VAPVANEN   = 0xD2, // Enable VAP/VAN signal output
  RDID1      = 0xDA, // Read ID1
  RDID2      = 0xDB, // Read ID2
  RDID3      = 0xDC, // Read ID3
  CMD2EN     = 0xDF, // Command 2 enable
  PVGAMCTRL  = 0xE0, // Positive voltage gamma control
  NVGAMCTRL  = 0xE1, // Negative voltage gamma control
  DGMLUTR    = 0xE2, // Digital gamma look-up table for red
  DGMLUTB    = 0xE3, // Digital gamma look-up table for blue
  GATECTRL   = 0xE4, // Gate control
  SPI2EN     = 0xE7, // SPI2 enable
  PWCTRL2    = 0xE8, // Power control 2
  EQCTRL     = 0xE9, // Equalize time control
  PROMCTRL   = 0xEC, // Program mode control
  PROMEN     = 0xFA, // Program mode enable
  NVMSET     = 0xFC, // NVM setting
  PROMACT    = 0xFE  // Program action
}

#[repr(u8)]
#[allow(dead_code)]
pub enum DisplayColorMode {
  BPP12   = 0b00000011, // 12 bits/pixel
  BPP16   = 0b00000101, // 16 bits/pixel -> 0bRRRRRGGG_GGGBBBBB
  BPP18   = 0b00000110, // 18 bits/pixel -> 0bRRRRRR00_GGGGGG00_BBBBBB00
  BPP16M  = 0b00000111, // 16M truncated -> 0bRRRRR000_GGGGGG00_BBBBB000
  RGB65K  = 0b01010000, // 65K of RGB interface
  RGB262K = 0b01100000  // 262K of RGB interface
}

impl BitOr for DisplayColorMode {
  type Output = u8;

  fn bitor(self, rhs: Self) -> Self::Output {
    self as u8 | rhs as u8
  }
}

#[repr(u8)]
#[derive(PartialEq)]
#[allow(dead_code)]
pub enum DisplayColorModeBPP {
  BPP12,
  BPP16,  // 16 bits/pixel -> 0bRRRRRGGG_GGGBBBBB
  BPP18,  // 18 bits/pixel -> 0bRRRRRR00_GGGGGG00_BBBBBB00
  BPP16M, // 16M truncated -> 0bRRRRR000_GGGGGG00_BBBBB000
  UNKNOWN 
}

#[repr(u8)]
#[allow(dead_code)]
pub enum DisplayMADCTL {
  MH  = 0b00000100, // Display data latch order
  RGB = 0b00001000, // RGB/BGR order
  ML  = 0b00010000, // Line address order
  MV  = 0b00100000, // Page/column order
  MX  = 0b01000000, // Column address order
  MY  = 0b10000000  // Page address order
}

impl BitOr for DisplayMADCTL {
  type Output = u8;

  fn bitor(self, rhs: Self) -> Self::Output {
    self as u8 | rhs as u8
  }
}

#[repr(u8)]
#[allow(dead_code)]
pub enum DisplayRotation {
  Portrait          = 0b00000000,
  Landscape         = 0b01100000, // DisplayMADCTL::MV | DisplayMADCTL::MX
  InvertedLandscape = 0b10100000, // DisplayMADCTL::MV | DisplayMADCTL::MY
  InvertedPortrait  = 0b11000000  // DisplayMADCTL::MX | DisplayMADCTL::MY
}

pub struct DisplayPinsData {
  pub backlight   : Pin<Gpio13, <Gpio13 as PinId>::Reset>,
  pub chip_select : Pin<Gpio9 , <Gpio9  as PinId>::Reset>,
  pub data_command: Pin<Gpio8 , <Gpio8  as PinId>::Reset>,
  pub reset       : Pin<Gpio12, <Gpio12 as PinId>::Reset>,
  pub spi_clock   : Pin<Gpio10, <Gpio10 as PinId>::Reset>,
  pub spi_mosi    : Pin<Gpio11, <Gpio11 as PinId>::Reset>
}

struct DisplayPins {
  _spi_clock  : Pin<Gpio10, FunctionSpi>,
  _spi_mosi   : Pin<Gpio11, FunctionSpi>,
  backlight   : Pin<Gpio13, PushPullOutput>,
  chip_select : Pin<Gpio9 , PushPullOutput>,
  data_command: Pin<Gpio8 , PushPullOutput>,
  reset       : Pin<Gpio12, PushPullOutput>
}

struct DisplayTextData {
  background_color: Option<u32>,
  foreground_color: u32,
  pixel_height    : u16,
  pixel_width     : u16
}

pub struct DisplaySpiData<'a> {
  pub baudrate  : HertzU32,
  pub clock     : HertzU32,
  pub mode      : &'a Mode,
  pub peripheral: SPI1,
  pub resets    : &'a mut RESETS
}

pub struct Display {
  bpp   : DisplayColorModeBPP,
  height: u16,
  pins  : DisplayPins,
  spi   : Spi<Enabled, SPI1, 8>,
  text  : DisplayTextData,
  width : u16
}

#[allow(dead_code)]
impl Display {
  pub fn new(width: u16, height: u16, bpp: DisplayColorModeBPP, rotation: DisplayRotation, pins_data: DisplayPinsData, spi_data: DisplaySpiData, delay: &mut Delay) -> Self {
    let mut display = Self {
      bpp: DisplayColorModeBPP::UNKNOWN,
      height: height,
      pins  : DisplayPins {
        _spi_clock  : pins_data.spi_clock.into_mode::<FunctionSpi>(),
        _spi_mosi   : pins_data.spi_mosi.into_mode::<FunctionSpi>(),
        backlight   : pins_data.backlight.into_push_pull_output(),
        chip_select : pins_data.chip_select.into_push_pull_output(),
        data_command: pins_data.data_command.into_push_pull_output(),
        reset       : pins_data.reset.into_push_pull_output()
      },
      spi   : Spi::new(spi_data.peripheral).init(
        spi_data.resets,
        spi_data.clock,
        spi_data.baudrate,
        spi_data.mode
      ),
      text  : DisplayTextData {
        background_color: None,
        foreground_color: 0xFFFFFFFF,
        pixel_height    : 1,
        pixel_width     : 1
      },
      width : width
    };

    display.hard_reset(delay);
    display.soft_reset(delay);
    display.set_sleep_mode(false);
    display.set_bpp(bpp);
    display.set_rotation(rotation);
    display.set_inversion_mode(true);
    display.set_normal_mode();
    display.fill(0);
    display.set_backlight(true);
    display.set_display(true);

    display
  }

  pub fn draw_solid_rect(&mut self, mut x: u16, mut y: u16, mut width: u16, mut height: u16, color: u32) {
    if self.bpp == DisplayColorModeBPP::UNKNOWN {
      return;
    }

    if x >= self.width {
      x = self.width - 1;
    }

    if y >= self.height {
      y = self.height - 1;
    }

    if x + width >= self.width {
      width = self.width - x;
    }

    if y + height >= self.height {
      height = self.height - y;
    }

    self.set_window(x, y, x + width - 1, y + height - 1);

    let mut bytes_per_pixel = 0u8;
    let buf = &mut [0, 0, 0];
    let pixels_count = width * height;

    if self.bpp == DisplayColorModeBPP::BPP12 {
      // Nothing yet
    }
    else if self.bpp == DisplayColorModeBPP::BPP16 {
      bytes_per_pixel = 2;

      let color_hi = ((color >> 8) & 0xFF) as u8;
      let color_lo = ((color >> 0) & 0xFF) as u8;

      buf[0] = color_hi;
      buf[1] = color_lo;
    }
    else if self.bpp == DisplayColorModeBPP::BPP18 || self.bpp == DisplayColorModeBPP::BPP16M {
      bytes_per_pixel = 3;

      let color_r = ((color >> 16) & 0xFF) as u8;
      let color_g = ((color >> 8 ) & 0xFF) as u8;
      let color_b = ((color >> 0 ) & 0xFF) as u8;

      buf[0] = color_r;
      buf[1] = color_g;
      buf[2] = color_b;
    }

    for _ in 0..pixels_count {
      self.send_data(&buf[0..bytes_per_pixel as usize]);
    }
    // let chunks = pixels_count / BUFFER_SIZE;
    // let rest = pixels_count % BUFFER_SIZE;

    // let buf = &mut [0u8; BUFFER_SIZE as usize * 2];

    // for i in 0..BUFFER_SIZE {
    //   buf[i as usize * 2    ] = color_hi;
    //   buf[i as usize * 2 + 1] = color_lo;
    // }

    // for _ in 0..chunks {
    //   self.send_data(buf);
    // }

    // if rest > 0 {
    //   self.send_data(&buf[0..rest as usize * 2]);
    // }
  }

  pub fn fill(&mut self, color: u32) {
    self.draw_solid_rect(0, 0, self.width, self.height, color);
  }

  pub fn hard_reset(&mut self, delay: &mut Delay) {
    self.pins.chip_select.set_low().unwrap();

    self.pins.reset.set_high().unwrap();

    delay.delay_ms(50);

    self.pins.reset.set_low().unwrap();

    delay.delay_ms(50);

    self.pins.reset.set_high().unwrap();

    delay.delay_ms(150);

    self.pins.chip_select.set_high().unwrap();
  }

  pub fn send_command(&mut self, command: DisplayCommand) {
    self.pins.chip_select.set_low().unwrap();

    self.pins.data_command.set_low().unwrap();

    self.spi.write(&[command as u8]).unwrap();

    self.pins.chip_select.set_high().unwrap();
  }

  pub fn send_data(&mut self, data: &[u8]) {
    self.pins.chip_select.set_low().unwrap();

    self.pins.data_command.set_high().unwrap();

    self.spi.write(data).unwrap();

    self.pins.chip_select.set_high().unwrap();
  }

  pub fn set_backlight(&mut self, on: bool) {
    if on {
      self.pins.backlight.set_high().unwrap();
    }
    else {
      self.pins.backlight.set_low().unwrap();
    }
  }

  pub fn set_bpp(&mut self, bpp: DisplayColorModeBPP) {
    self.bpp = bpp;

    let color_mode: u8;

    match self.bpp {
      DisplayColorModeBPP::BPP12  => color_mode = DisplayColorMode::RGB65K  | DisplayColorMode::BPP12,
      DisplayColorModeBPP::BPP16  => color_mode = DisplayColorMode::RGB65K  | DisplayColorMode::BPP16,
      DisplayColorModeBPP::BPP16M => color_mode = DisplayColorMode::RGB65K  | DisplayColorMode::BPP16M,
      DisplayColorModeBPP::BPP18  => color_mode = DisplayColorMode::RGB262K | DisplayColorMode::BPP12,
      _                           => color_mode = DisplayColorMode::RGB65K  | DisplayColorMode::BPP16
    }

    self.set_color_mode(color_mode);
  }

  fn set_color_mode(&mut self, mode: u8) {
    self.send_command(DisplayCommand::COLMOD);

    self.send_data(&[mode]);
  }

  pub fn set_columns(&mut self, start: u16, end: u16) {
    if start > end || end >= self.width {
      return;
    }

    let start_hi = (start >> 8) as u8;
    let start_lo = (start & 0xFF) as u8;

    let end_hi = (end >> 8) as u8;
    let end_lo = (end & 0xFF) as u8;

    self.send_command(DisplayCommand::CASET);

    self.send_data(&[start_hi, start_lo, end_hi, end_lo]);
  }

  pub fn set_display(&mut self, on: bool) {
    if on {
      self.send_command(DisplayCommand::DISPON);
    }
    else {
      self.send_command(DisplayCommand::DISPOFF);
    }
  }

  pub fn set_inversion_mode(&mut self, on: bool) {
    if on {
      self.send_command(DisplayCommand::INVON);
    }
    else {
      self.send_command(DisplayCommand::INVOFF);
    }
  }

  pub fn set_normal_mode(&mut self) {
    self.send_command(DisplayCommand::NORON);
  }

  pub fn set_rotation(&mut self, rotation: DisplayRotation) {
    self.send_command(DisplayCommand::MADCTL);

    self.send_data(&[rotation as u8]);
  }

  pub fn set_sleep_mode(&mut self, on: bool) {
    if on {
      self.send_command(DisplayCommand::SLPIN);
    }
    else {
      self.send_command(DisplayCommand::SLPOUT);
    }
  }

  pub fn set_rows(&mut self, start: u16, end: u16) {
    if start > end || end >= self.height {
      return;
    }

    let start_hi = (start >> 8) as u8;
    let start_lo = (start & 0xFF) as u8;

    let end_hi = (end >> 8) as u8;
    let end_lo = (end & 0xFF) as u8;

    self.send_command(DisplayCommand::RASET);

    self.send_data(&[start_hi, start_lo, end_hi, end_lo]);
  }

  pub fn set_text_background_color(&mut self, color: Option<u32>) {
    self.text.background_color = color;
  }

  pub fn set_text_foreground_color(&mut self, color: u32) {
    self.text.foreground_color = color;
  }

  pub fn set_text_pixel_height(&mut self, pixel_height: u16) {
    self.text.pixel_height = pixel_height;
  }

  pub fn set_text_pixel_width(&mut self, pixel_width: u16) {
    self.text.pixel_width = pixel_width;
  }

  pub fn set_window(&mut self, start_x: u16, start_y: u16, end_x: u16, end_y: u16) {
    self.set_columns(start_x, end_x);

    self.set_rows(start_y, end_y);

    self.send_command(DisplayCommand::RAMWR);
  }

  pub fn soft_reset(&mut self, delay: &mut Delay) {
    self.send_command(DisplayCommand::SWRESET);

    delay.delay_ms(150);
  }

  pub fn draw_text(&mut self, x: u16, y: u16, text: String<31>) {
    let char_width = 8 * self.text.pixel_width as u16;
    let char_height = 8 * self.text.pixel_height as u16;

    let mut render_x = x;
    let mut render_y = y;

    for c in text.chars() {
      let font_index = c as usize;

      if font_index == 0x0A {
        render_x = x;
        render_y += char_height;

        continue;
      }

      let char = FONT[font_index];

      for char_row in char {
        for char_column in (0..=7).rev() {
          if char_row & (1 << char_column) != 0 {
            self.draw_solid_rect(render_x, render_y, self.text.pixel_width, self.text.pixel_height, self.text.foreground_color);
          }
          else if self.text.background_color != None {
            self.draw_solid_rect(render_x, render_y, self.text.pixel_width, self.text.pixel_height, self.text.background_color.unwrap());
          }

          render_x += self.text.pixel_width;
        }

        render_x -= char_width;
        render_y += self.text.pixel_height;
      }

      render_x += char_width;
      render_y -= char_height;
    }
  }

  pub fn height(&self) -> u16 {
    self.height
  }

  pub fn width(&self) -> u16 {
    self.width
  }
}
