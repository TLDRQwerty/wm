extern crate libc;
extern crate x11;

use libc::{c_int, c_uint};
use std::ffi::OsStr;
use std::mem::zeroed;
use std::process::Command;
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
    window_attributes: XWindowAttributes,
}

impl WindowManager {
    fn new() -> Self {
        let display = unsafe { XOpenDisplay(0x0 as *const i8) };
        if display.is_null() {
            panic!("Failed to find display");
        }

        let window: Window = unsafe { XDefaultRootWindow(display) };
        let mut window_attributes: XWindowAttributes = unsafe { zeroed() };
        unsafe { XGetWindowAttributes(display, window, &mut window_attributes) };
        return Self {
            display,
            window,
            window_attributes,
        };
    }
}

fn spawn_process(process: &OsStr) {
    match Command::new(process).spawn() {
        Err(e) => eprintln!("couldn't spawn: {}", e.to_string()),
        _ => {}
    };
}

fn main() {
    let mut start: XButtonEvent = unsafe { zeroed() };
    let mut attr: XWindowAttributes = unsafe { zeroed() };
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

    loop {
        unsafe {
            XNextEvent(wm.display, &mut event);
            XGetInputFocus(wm.display, &mut wm.window, &mut revert_to);

            match event.get_type() {
                KeyPress => {
                    let ev: XKeyEvent = From::from(event);

                    if ev.subwindow != 0 {
                        XRaiseWindow(wm.display, ev.subwindow);
                    }

                    match XKeycodeToKeysym(
                        wm.display,
                        event.key.keycode.try_into().unwrap(),
                        0 as c_int,
                    ) as c_uint
                    {
                        XK_q => {
                            XDestroyWindow(wm.display, wm.window);
                        }
                        XK_Return => {
                            spawn_process(OsStr::new("kitty"));
                        }
                        XK_d => {
                            spawn_process(OsStr::new("dmenu_run"));
                        }
                        XK_space => {
                            match Command::new("rofi").args(&["-show", "run"]).spawn() {
                                Err(e) => eprintln!("couldn't spawn: {}", e.to_string()),
                                _ => {}
                            };
                        }
                        XK_BackSpace => {
                            XCloseDisplay(wm.display);
                        }
                        _ => {}
                    }
                }
                ButtonPress => {
                    let xbutton: XButtonEvent = From::from(event);
                    if xbutton.subwindow != 0 {
                        XGetWindowAttributes(wm.display, xbutton.subwindow, &mut attr);
                        start = xbutton;
                        XRaiseWindow(wm.display, xbutton.subwindow);
                    }
                }
                MotionNotify => {
                    let ev: XMotionEvent = From::from(event);
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
                ButtonRelease => {
                    let ev: XButtonReleasedEvent = From::from(event);
                    start.subwindow = 0;
                }
                MapRequest => {
                    let ev: XMapRequestEvent = From::from(event);
                    println!("map request -> {:?}", ev);
                }
                ConfigureRequest => {
                    let ev: XConfigureRequestEvent = From::from(event);
                    println!("configure request request -> {:?}", ev);
                }
                Expose => {
                    let ev: XExposeEvent = From::from(event);
                    println!("expose request -> {:?}", ev);
                }
                ClientMessage => {
                    let ev: XExposeEvent = From::from(event);
                    println!("client message -> {:?}", ev);
                }
                CreateNotify => {
                    let ev: XExposeEvent = From::from(event);
                    println!("create notify -> {:?}", ev);
                }
                PropertyNotify => {
                    let ev: XPropertyEvent = From::from(event);
                    println!("create notify -> {:?}", ev);
                }
                _ => {
                    println!("Unhandled Event {:?} \n {:?} \n", event.get_type(), event);
                }
            };
        }
    }
}
