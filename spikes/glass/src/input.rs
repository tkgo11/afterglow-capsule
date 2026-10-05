//! SPIKE ONLY: route synthetic input through Windows SendInput. Driver sends
//! never increment renderer input counters. Only real winit events acknowledge.
use std::time::{Duration, Instant};
use winit::{dpi::PhysicalPosition, window::Window};

#[derive(Default)]
pub struct InputDriver {
    pending_pointer: Option<(Instant, PhysicalPosition<f64>)>,
    pending_key: Option<Instant>,
    last_send: Option<Instant>,
    step: u32,
}
impl InputDriver {
    pub fn acknowledge_pointer(
        &mut self,
        position: PhysicalPosition<f64>,
        now: Instant,
    ) -> Option<f64> {
        let (sent, expected) = self.pending_pointer?;
        if (position.x - expected.x).abs() > 2.0 || (position.y - expected.y).abs() > 2.0 {
            return None;
        }
        self.pending_pointer = None;
        Some(now.duration_since(sent).as_secs_f64() * 1000.0)
    }
    pub fn acknowledge_key(&mut self, now: Instant) -> Option<f64> {
        self.pending_key
            .take()
            .map(|sent| now.duration_since(sent).as_secs_f64() * 1000.0)
    }
    pub fn pump(&mut self, window: &Window, now: Instant) -> Result<(), String> {
        if self
            .pending_pointer
            .is_some_and(|(sent, _)| now.duration_since(sent) > Duration::from_secs(2))
            || self
                .pending_key
                .is_some_and(|sent| now.duration_since(sent) > Duration::from_secs(2))
        {
            return Err("SendInput produced no matching received event within two seconds".into());
        }
        if self.pending_pointer.is_some()
            || self.pending_key.is_some()
            || self
                .last_send
                .is_some_and(|sent| now.duration_since(sent) < Duration::from_millis(200))
        {
            return Ok(());
        }
        let size = window.inner_size();
        let position = PhysicalPosition::new(
            f64::from(size.width)
                * (if self.step.is_multiple_of(2) {
                    0.45
                } else {
                    0.55
                }),
            f64::from(size.height) * 0.5,
        );
        let position = send_input(window, position)?;
        self.pending_pointer = Some((now, position));
        self.pending_key = Some(now);
        self.last_send = Some(now);
        self.step += 1;
        Ok(())
    }
}

#[cfg(any(windows, test))]
fn distinct_target(
    requested: PhysicalPosition<f64>,
    current: PhysicalPosition<f64>,
    width: u32,
) -> PhysicalPosition<f64> {
    if (requested.x - current.x).abs() <= 2.0 && (requested.y - current.y).abs() <= 2.0 {
        PhysicalPosition::new(
            f64::from(width)
                * (if requested.x < f64::from(width) * 0.5 {
                    0.55
                } else {
                    0.45
                }),
            requested.y,
        )
    } else {
        requested
    }
}

#[cfg(windows)]
fn send_input(
    window: &Window,
    position: PhysicalPosition<f64>,
) -> Result<PhysicalPosition<f64>, String> {
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
            MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT, SendInput,
            VK_F8,
        },
        WindowsAndMessaging::{
            GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowThreadProcessId,
            SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        },
    };
    let origin = window
        .inner_position()
        .map_err(|e| format!("cannot locate client input area: {e}"))?;
    // SAFETY: read-only desktop APIs and fixed initialized INPUT structures. No
    // key shortcuts/clicks or OS setting changes. Only send while our process
    // owns the foreground window; this is not compiled into production Viewer.
    unsafe {
        let foreground = GetForegroundWindow();
        let mut process = 0;
        GetWindowThreadProcessId(foreground, &mut process);
        if process != std::process::id() {
            return Err("input driver refused to send to another foreground process".into());
        }
        let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        if width <= 1 || height <= 1 {
            return Err("invalid virtual desktop dimensions".into());
        }
        let mut cursor = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor) == 0 {
            return Err("cannot determine physical cursor position before input-driving".into());
        }
        let current_client = PhysicalPosition::new(
            f64::from(cursor.x - origin.x),
            f64::from(cursor.y - origin.y),
        );
        // Windows can coalesce unchanged mouse movement. Ensure the requested
        // target differs from the actual OS cursor, including the first send
        // after a preceding cell ended at the same interior target.
        let position = distinct_target(position, current_client, window.inner_size().width);
        let screen_x = f64::from(origin.x) + position.x;
        let screen_y = f64::from(origin.y) + position.y;
        if screen_x < f64::from(left)
            || screen_x >= f64::from(left + width)
            || screen_y < f64::from(top)
            || screen_y >= f64::from(top + height)
        {
            return Err("input target lies outside the physical desktop".into());
        }
        let mouse = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: ((screen_x - f64::from(left)) * 65535.0 / f64::from(width - 1)).round()
                        as i32,
                    dy: ((screen_y - f64::from(top)) * 65535.0 / f64::from(height - 1)).round()
                        as i32,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let key = |flags| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_F8,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let events = [mouse, key(0), key(KEYEVENTF_KEYUP)];
        if SendInput(
            events.len() as u32,
            events.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        ) != events.len() as u32
        {
            let failure = std::io::Error::last_os_error();
            // If a partially accepted batch included key-down, release F8 before
            // failing. This cleanup is not recorded as renderer receipt.
            let release = key(KEYEVENTF_KEYUP);
            SendInput(1, &release, std::mem::size_of::<INPUT>() as i32);
            return Err(format!("SendInput was rejected: {failure}"));
        }
        Ok(position)
    }
}
#[cfg(not(windows))]
fn send_input(
    _window: &Window,
    _position: PhysicalPosition<f64>,
) -> Result<PhysicalPosition<f64>, String> {
    Err(
        "--drive-input requires Windows SendInput; no synthetic renderer events are substituted"
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_cell_target_moves_from_actual_cursor() {
        let target = PhysicalPosition::new(432.0, 320.0);
        assert_eq!(
            distinct_target(target, target, 960),
            PhysicalPosition::new(528.0, 320.0)
        );
        assert_eq!(
            distinct_target(target, PhysicalPosition::new(500.0, 320.0), 960),
            target
        );
    }
    #[test]
    fn sends_are_not_received_events_and_only_matching_events_acknowledge() {
        let now = Instant::now();
        let expected = PhysicalPosition::new(10.0, 20.0);
        let mut driver = InputDriver {
            pending_pointer: Some((now, expected)),
            pending_key: Some(now),
            ..Default::default()
        };
        assert!(
            driver
                .acknowledge_pointer(PhysicalPosition::new(40.0, 20.0), now)
                .is_none()
        );
        assert_eq!(driver.acknowledge_pointer(expected, now), Some(0.0));
        assert!(driver.acknowledge_pointer(expected, now).is_none());
        assert_eq!(driver.acknowledge_key(now), Some(0.0));
        assert!(driver.acknowledge_key(now).is_none());
    }
}
