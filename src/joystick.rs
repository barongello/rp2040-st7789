use core::ops::{
  BitAnd,
  BitOr,
  BitOrAssign
};
use embedded_hal::digital::v2::InputPin;
use rp2040_hal::gpio::{
  Pin,
  PinId,
  PullUpInput,
  bank0::{
    Gpio2,
    Gpio3,
    Gpio15,
    Gpio16,
    Gpio17,
    Gpio18,
    Gpio19,
    Gpio20,
    Gpio21
  }
};

#[repr(u16)]
#[allow(dead_code)]
pub enum JoystickButton {
  A     = 0b0000000000000001, // 0x0001
  B     = 0b0000000000000010, // 0x0002
  X     = 0b0000000000000100, // 0x0004
  Y     = 0b0000000000001000, // 0x0008
  UP    = 0b0000000000010000, // 0x0010
  DOWN  = 0b0000000000100000, // 0x0020
  LEFT  = 0b0000000001000000, // 0x0040
  RIGHT = 0b0000000010000000, // 0x0080
  CTRL  = 0b0000000100000000  // 0x0100
}

impl BitAnd for JoystickButton {
  type Output = u16;

  fn bitand(self, rhs: Self) -> Self::Output {
    self as u16 & rhs as u16
  }
}

impl BitOr for JoystickButton {
  type Output = u16;

  fn bitor(self, rhs: Self) -> Self::Output {
    self as u16 | rhs as u16
  }
}

impl BitOr<JoystickButton> for u16 {
  type Output = u16;

  fn bitor(self, rhs: JoystickButton) -> Self::Output {
    self | rhs as u16
  }
}

impl BitOrAssign<JoystickButton> for u16 {
  fn bitor_assign(&mut self, rhs: JoystickButton) {
      *self |= rhs as u16
  }
}

pub struct JoystickButtonsData {
  pub a    : Pin<Gpio15, <Gpio15 as PinId>::Reset>,
  pub b    : Pin<Gpio17, <Gpio17 as PinId>::Reset>,
  pub x    : Pin<Gpio19, <Gpio19 as PinId>::Reset>,
  pub y    : Pin<Gpio21, <Gpio21 as PinId>::Reset>,
  pub up   : Pin<Gpio2 , <Gpio2  as PinId>::Reset>,
  pub down : Pin<Gpio18, <Gpio18 as PinId>::Reset>,
  pub left : Pin<Gpio16, <Gpio16 as PinId>::Reset>,
  pub right: Pin<Gpio20, <Gpio20 as PinId>::Reset>,
  pub ctrl : Pin<Gpio3 , <Gpio3  as PinId>::Reset>
}

struct JoystickButtons {
  a    : Pin<Gpio15, PullUpInput>,
  b    : Pin<Gpio17, PullUpInput>,
  x    : Pin<Gpio19, PullUpInput>,
  y    : Pin<Gpio21, PullUpInput>,
  up   : Pin<Gpio2 , PullUpInput>,
  down : Pin<Gpio18, PullUpInput>,
  left : Pin<Gpio16, PullUpInput>,
  right: Pin<Gpio20, PullUpInput>,
  ctrl : Pin<Gpio3 , PullUpInput>
}

type JoystickState = u16;

struct JoystickStates {
  current: JoystickState,
  old    : JoystickState
}

pub struct Joystick {
  buttons: JoystickButtons,
  states : JoystickStates
}

#[allow(dead_code)]
impl Joystick {
  pub fn new(buttons: JoystickButtonsData) -> Self {
    Self {
      buttons: JoystickButtons {
        a    : buttons.a    .into_pull_up_input(),
        b    : buttons.b    .into_pull_up_input(),
        x    : buttons.x    .into_pull_up_input(),
        y    : buttons.y    .into_pull_up_input(),
        up   : buttons.up   .into_pull_up_input(),
        down : buttons.down .into_pull_up_input(),
        left : buttons.left .into_pull_up_input(),
        right: buttons.right.into_pull_up_input(),
        ctrl : buttons.ctrl .into_pull_up_input()
      },
      states : JoystickStates {
        current: 0x0000,
        old    : 0x0000
      }
    }
  }

  pub fn update(&mut self) {
    self.states.old = self.states.current;

    let mut current_state: JoystickState = 0x0000;

    if self.buttons.a.is_low().unwrap() {
      current_state |= JoystickButton::A as u16;
    }

    if self.buttons.b.is_low().unwrap() {
      current_state |= JoystickButton::B as u16;
    }

    if self.buttons.x.is_low().unwrap() {
      current_state |= JoystickButton::X as u16;
    }

    if self.buttons.y.is_low().unwrap() {
      current_state |= JoystickButton::Y as u16;
    }

    if self.buttons.up.is_low().unwrap() {
      current_state |= JoystickButton::UP as u16;
    }

    if self.buttons.down.is_low().unwrap() {
      current_state |= JoystickButton::DOWN as u16;
    }

    if self.buttons.left.is_low().unwrap() {
      current_state |= JoystickButton::LEFT as u16;
    }

    if self.buttons.right.is_low().unwrap() {
      current_state |= JoystickButton::RIGHT as u16;
    }

    if self.buttons.ctrl.is_low().unwrap() {
      current_state |= JoystickButton::CTRL as u16;
    }

    self.states.current = current_state;
  }

  pub fn is_active(&self, button: JoystickButton) -> bool {
    let button_u16 = button as u16;

    self.states.current & button_u16 == button_u16
  }

  pub fn just_pressed(&self, button: JoystickButton) -> bool {
    let button_u16 = button as u16;

    self.states.old & button_u16 == 0 && self.states.current & button_u16 == button_u16
  }

  pub fn just_released(&self, button: JoystickButton) -> bool {
    let button_u16 = button as u16;

    self.states.old & button_u16 == button_u16 && self.states.current & button_u16 == 0
  }

  pub fn is_hold(&self, button: JoystickButton) -> bool {
    let button_u16 = button as u16;

    self.states.old & button_u16 == button_u16 && self.states.current & button_u16 == button_u16
  }

  pub fn is_any_active(&self, buttons: Option<u16>) -> bool {
    let buttons_value = buttons.unwrap_or(0x01FF);

    self.states.current & buttons_value != 0
  }

  pub fn just_pressed_any(&self) -> bool {
    self.just_pressed(JoystickButton::A) ||
    self.just_pressed(JoystickButton::B) ||
    self.just_pressed(JoystickButton::X) ||
    self.just_pressed(JoystickButton::Y) ||
    self.just_pressed(JoystickButton::UP) ||
    self.just_pressed(JoystickButton::DOWN) ||
    self.just_pressed(JoystickButton::LEFT) ||
    self.just_pressed(JoystickButton::RIGHT) ||
    self.just_pressed(JoystickButton::CTRL)
  }

  pub fn just_released_any(&self) -> bool {
    self.just_released(JoystickButton::A) ||
    self.just_released(JoystickButton::B) ||
    self.just_released(JoystickButton::X) ||
    self.just_released(JoystickButton::Y) ||
    self.just_released(JoystickButton::UP) ||
    self.just_released(JoystickButton::DOWN) ||
    self.just_released(JoystickButton::LEFT) ||
    self.just_released(JoystickButton::RIGHT) ||
    self.just_released(JoystickButton::CTRL)
  }
}
