extern crate libc;
extern crate x11;

use libc::{c_int, c_uint};
use std::ffi::OsStr;
use std::process::Command;
use std::mem::zeroed;
use x11::{keysym::*, xlib::*};

fn max(a: c_int, b: c_int) -> c_uint {
    if a > b {
        a as c_uint
    } else {
        b as c_uint
    }
}

struct WindowManager {
    display: *mut Display,
    window: Window,
}

impl WindowManager {
    fn new() -> Self {
        let display = unsafe { XOpenDisplay(0x0 as *const i8) };
        if display.is_null() {
            panic!("Failed to find display");
        }

        let window: Window = unsafe { XDefaultRootWindow(display) };
        return Self { display, window };
    }
}

fn spawn_process(process: &OsStr) {
    match Command::new(process).spawn() {
        Err(e) => eprintln!("couldn't spawn: {}", e.to_string()),
        _ => {}
    };
}

fn main() {
    let mut arg0 = 0x0 as i8;
    let mut attr: XWindowAttributes = unsafe { zeroed() };
    let mut start: XButtonEvent = unsafe { zeroed() };
    let mut revert_to: i32 = 0;
    //let mut cursor: Cursor = unsafe { zeroed() };

    let mut wm = WindowManager::new();

    unsafe {
        let shortcuts: Vec<c_uint> = vec![XK_d, XK_q, XK_Return, XK_space, XK_BackSpace];
        for key in shortcuts {
            XGrabKey(
                wm.display,
                XKeysymToKeycode(wm.display, key.into()) as c_int,
                Mod4Mask,
                wm.window,
                true as c_int,
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        XGrabButton(
            wm.display,
            1 as c_uint,
            Mod4Mask,
            wm.window,
            true as c_int,
            (ButtonPressMask | ButtonReleaseMask | PointerMotionMask) as c_uint,
            GrabModeAsync,
            GrabModeAsync,
            0,
            0,
        );
        XGrabButton(
            wm.display,
            3 as c_uint,
            Mod4Mask,
            wm.window,
            true as c_int,
            (ButtonPressMask | ButtonReleaseMask | PointerMotionMask) as c_uint,
            GrabModeAsync,
            GrabModeAsync,
            0,
            0,
        );
    };

    start.subwindow = 0;
    let mut event: XEvent = unsafe { zeroed() };

    let mut window_attributes = unsafe { zeroed() };

    unsafe {
        XGetWindowAttributes(wm.display, wm.window, &mut window_attributes);
    };

    println!("{:?}", window_attributes);

    loop {
        unsafe {
            XNextEvent(wm.display, &mut event);
            XGetInputFocus(wm.display, &mut wm.window, &mut revert_to);

            match event.get_type() {
                x11::xlib::KeyPress => {
                    let xkey: XKeyEvent = From::from(event);
                    println!("{:?}", xkey);
                    if xkey.subwindow != 0 {
                        XRaiseWindow(wm.display, xkey.subwindow);

                        // Close window with mod+q
                        if event.key.keycode
                            == XKeysymToKeycode(wm.display, x11::keysym::XK_q.into()).into()
                        {
                            XDestroyWindow(wm.display, wm.window);
                        }
                    }

                    // Open a terminal with mod+enter
                    if event.key.keycode == XKeysymToKeycode(wm.display, XK_Return.into()).into() {
                        spawn_process(OsStr::new("kitty"));
                    }

                    // Open dmenu with mod+d
                    if event.key.keycode == XKeysymToKeycode(wm.display, XK_d.into()).into() {
                        spawn_process(OsStr::new("dmenu_run"));
                    }

                    // Open rofi with mod+space
                    if event.key.keycode == XKeysymToKeycode(wm.display, XK_space.into()).into() {
                        match Command::new("rofi").args(&["-show", "run"]).spawn() {
                            Err(e) => eprintln!("couldn't spawn: {}", e.to_string()),
                            _ => {}
                        };
                    }

                    // Close r9wm with mod+backspace
                    if event.key.keycode == XKeysymToKeycode(wm.display, XK_BackSpace.into()).into()
                    {
                        XCloseDisplay(wm.display);
                    }
                }
                x11::xlib::ButtonPress => {
                    let xbutton: XButtonEvent = From::from(event);
                    if xbutton.subwindow != 0 {
                        XGetWindowAttributes(wm.display, xbutton.subwindow, &mut attr);
                        start = xbutton;
                        XRaiseWindow(wm.display, xbutton.subwindow);
                    }
                }
                x11::xlib::MotionNotify => {
                    if start.subwindow != 0 {
                        //cursor = XCreateFontCursor(display, 58);
                        //XDefineCursor(display, start.subwindow, cursor);
                        let xbutton: XButtonEvent = From::from(event);
                        let xdiff: c_int = xbutton.x_root - start.x_root;
                        let ydiff: c_int = xbutton.y_root - start.y_root;
                        XMoveResizeWindow(
                            wm.display,
                            start.subwindow,
                            attr.x + (if start.button == 1 { xdiff } else { 0 }),
                            attr.y + (if start.button == 1 { ydiff } else { 0 }),
                            max(1, attr.width + (if start.button == 3 { xdiff } else { 0 })),
                            max(1, attr.height + (if start.button == 3 { ydiff } else { 0 })),
                        );
                    }
                }
                x11::xlib::ButtonRelease => {
                    start.subwindow = 0;
                }
                _ => {}
            };
        }
    }
}
