#![no_std]
#![no_main]

mod display;
mod font;
mod joystick;
mod photos;

use cortex_m_rt::entry;
use defmt_rtt as _;
use display::{
  Display,
  DisplayColorModeBPP,
  DisplayPinsData,
  DisplayRotation,
  DisplaySpiData
};
use fugit::RateExtU32;
use heapless::String;
use joystick::{
  Joystick,
  JoystickButton,
  JoystickButtonsData
};
use panic_probe as _;
use rp2040_hal as hal;

use hal::{
  clocks::{
    init_clocks_and_plls,
    Clock
  },
  pac,
  watchdog::Watchdog,
  Sio,
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
  let mut pac = pac::Peripherals::take().unwrap();
  let core = pac::CorePeripherals::take().unwrap();
  let mut watchdog = Watchdog::new(pac.WATCHDOG);
  let sio = Sio::new(pac.SIO);

  let external_xtal_freq_hz = 12_000_000u32;
  let clocks = init_clocks_and_plls(
    external_xtal_freq_hz,
    pac.XOSC,
    pac.CLOCKS,
    pac.PLL_SYS,
    pac.PLL_USB,
    &mut pac.RESETS,
    &mut watchdog,
  )
  .ok()
  .unwrap();

  let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().raw());

  let pins = hal::gpio::Pins::new(
    pac.IO_BANK0,
    pac.PADS_BANK0,
    sio.gpio_bank0,
    &mut pac.RESETS,
  );

  let display_pins_data = DisplayPinsData {
    backlight   : pins.gpio13,
    chip_select : pins.gpio9,
    data_command: pins.gpio8,
    reset       : pins.gpio12,
    spi_clock   : pins.gpio10,
    spi_mosi    : pins.gpio11
  };

  let display_spi_data = DisplaySpiData {
    baudrate: 30.MHz(),
    clock: clocks.peripheral_clock.freq(),
    mode: &embedded_hal::spi::MODE_0,
    peripheral: pac.SPI1,
    resets: &mut pac.RESETS
  };

  let mut display = Display::new(
    240,
    240,
    DisplayColorModeBPP::BPP16,
    DisplayRotation::Landscape,
    display_pins_data,
    display_spi_data,
    &mut delay
  );

  let background_color: u32 = 0b00110_001101_00110;
  let foreground_color: u32 = 0b00000_101100_00000;

  display.fill(background_color);
  display.set_text_foreground_color(foreground_color);
  display.set_text_background_color(Some(background_color));
  display.set_text_pixel_height(2);
  display.set_text_pixel_width(2);

  display.set_window(0, 0, 239, 239);
  display.send_data(&photos::PHOTOS[0]);

  display.draw_text(5, 5, String::from("0"));

  let joystick_buttons_data = JoystickButtonsData {
    a    : pins.gpio15,
    b    : pins.gpio17,
    x    : pins.gpio19,
    y    : pins.gpio21,
    up   : pins.gpio2,
    down : pins.gpio18,
    left : pins.gpio16,
    right: pins.gpio20,
    ctrl : pins.gpio3
  };

  let mut joystick = Joystick::new(joystick_buttons_data);

  let mut x = 0u16;
  let mut y = 0u16;
  let mut w = 0u16;
  let mut h = 0u16;

  let mut photo: i8 = -1;
  let hidden_photo_index = 4;
  let mut ctrl_hold_counter = 0;
  let ctrl_hold_counter_threshold = 4_000_000;

  let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
  let mut time_old: u32 = timer.get_counter_low();
  let mut fps = 0;

  loop {
    let time_start: u32 = timer.get_counter_low();
    let time_diff = time_start - time_old;

    if time_diff >= 1000000 {
      let mut fps_str: String<31> = String::from("FPS:\n");

      fps_str.push_str(String::<31>::from(fps).as_str()).unwrap();

      display.draw_text(5, 5, fps_str);

      time_old = time_start;

      fps = 0;
    }

    joystick.update();

    if joystick.is_active(JoystickButton::A) {
      if w > 0 {
        w -= 1;
      }
    }

    if joystick.is_active(JoystickButton::B) {
      if w + x < display.width() - 1 {
        w += 1;
      }
    }

    if joystick.is_active(JoystickButton::X) {
      if h > 0 {
        h -= 1;
      }
    }

    if joystick.is_active(JoystickButton::Y) {
      if h + y < display.height() - 1 {
        h += 1;
      }
    }

    if joystick.is_active(JoystickButton::UP) {
      if y > 0 {
        y -= 1;
      }
    }

    if joystick.is_active(JoystickButton::DOWN) {
      if y + h < display.height() - 1 {
        y += 1;
      }
    }

    if joystick.is_active(JoystickButton::LEFT) {
      if x > 0 {
        x -= 1;
      }
    }

    if joystick.is_active(JoystickButton::RIGHT) {
      if x + w < display.width() - 1 {
        x += 1;
      }
    }

    if joystick.is_active(JoystickButton::CTRL) {
      let time_now: u32 = timer.get_counter_low() / 1000000;
      let time_str: String<31> = String::from(time_now);

      display.draw_text(5, 5, time_str);
    }

    if joystick.just_pressed(JoystickButton::CTRL) {
      photo += 1;
      photo %= photos::PHOTOS.len() as i8;

      if photo == hidden_photo_index {
        photo += 1;
        photo %= photos::PHOTOS.len() as i8;
      }

      display.set_window(0, 0, 239, 239);
      display.send_data(&photos::PHOTOS[photo as usize]);
    }
    else if joystick.is_hold(JoystickButton::CTRL) {
      ctrl_hold_counter += 1;

      if ctrl_hold_counter > ctrl_hold_counter_threshold {
        display.set_window(0, 0, 239, 239);
        display.send_data(&photos::PHOTOS[hidden_photo_index as usize]);
      }
    }
    else if joystick.just_released(JoystickButton::CTRL) {
      ctrl_hold_counter = 0;

      display.set_window(0, 0, 239, 239);
      display.send_data(&photos::PHOTOS[photo as usize]);
    }
    else if joystick.is_any_active(Some(JoystickButton::A | JoystickButton::B | JoystickButton::X | JoystickButton::Y | JoystickButton::UP | JoystickButton::DOWN | JoystickButton::LEFT | JoystickButton::RIGHT)) {
      display.fill(0);
      display.draw_solid_rect(x, y, w, h, 0b1111100000000000);
    }

    fps += 1;

    let time_end = timer.get_counter_low();

    delay.delay_us(1000000 / 15 - (time_end - time_start));
  }
}
