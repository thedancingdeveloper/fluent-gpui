use std::{
    cell::Cell,
    ffi::{c_uint, c_void},
};

use gpui_util::ResultExt;
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETHIGHCONTRAST, SPI_GETWHEELSCROLLCHARS, SPI_GETWHEELSCROLLLINES,
    SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

/// Windows settings pulled from SystemParametersInfo
/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow
#[derive(Default, Debug, Clone)]
pub(crate) struct WindowsSystemSettings {
    pub(crate) mouse_wheel_settings: MouseWheelSettings,
    pub(crate) high_contrast: Cell<bool>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct MouseWheelSettings {
    /// SEE: SPI_GETWHEELSCROLLCHARS
    pub(crate) wheel_scroll_chars: Cell<u32>,
    /// SEE: SPI_GETWHEELSCROLLLINES
    pub(crate) wheel_scroll_lines: Cell<u32>,
}

impl WindowsSystemSettings {
    pub(crate) fn new() -> Self {
        let mut settings = Self::default();
        settings.init();
        settings
    }

    fn init(&mut self) {
        self.mouse_wheel_settings.update();
        self.high_contrast.set(system_high_contrast());
    }

    pub(crate) fn update(&self, wparam: usize) {
        match SYSTEM_PARAMETERS_INFO_ACTION(wparam as u32) {
            SPI_GETHIGHCONTRAST => self.high_contrast.set(system_high_contrast()),
            SPI_GETWHEELSCROLLLINES | SPI_GETWHEELSCROLLCHARS => self.update_mouse_wheel_settings(),
            _ => {}
        }
    }

    fn update_mouse_wheel_settings(&self) {
        self.mouse_wheel_settings.update();
    }
}

pub(crate) fn system_high_contrast() -> bool {
    let mut info = HIGHCONTRASTW::default();
    info.cbSize = std::mem::size_of::<HIGHCONTRASTW>() as u32;
    let succeeded = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            info.cbSize,
            Some((&mut info) as *mut HIGHCONTRASTW as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        )
    }
    .is_ok();
    succeeded && info.dwFlags.contains(HCF_HIGHCONTRASTON)
}

impl MouseWheelSettings {
    fn update(&self) {
        self.update_wheel_scroll_chars();
        self.update_wheel_scroll_lines();
    }

    fn update_wheel_scroll_chars(&self) {
        let mut value = c_uint::default();
        let result = unsafe {
            SystemParametersInfoW(
                SPI_GETWHEELSCROLLCHARS,
                0,
                Some((&mut value) as *mut c_uint as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
            )
        };

        if result.log_err() != None && self.wheel_scroll_chars.get() != value {
            self.wheel_scroll_chars.set(value);
        }
    }

    fn update_wheel_scroll_lines(&self) {
        let mut value = c_uint::default();
        let result = unsafe {
            SystemParametersInfoW(
                SPI_GETWHEELSCROLLLINES,
                0,
                Some((&mut value) as *mut c_uint as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
            )
        };

        if result.log_err() != None && self.wheel_scroll_lines.get() != value {
            self.wheel_scroll_lines.set(value);
        }
    }
}
