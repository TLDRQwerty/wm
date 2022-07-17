use std::cmp::max;
use std::process::Command;

use x11rb::connection::Connection;
use x11rb::cookie::Cookie;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event::{ButtonPress, ClientMessage, KeyPress, MotionNotify};
use x11rb::rust_connection::{ConnectionError, DefaultStream, RustConnection};
use x11rb::COPY_DEPTH_FROM_PARENT;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let root = conn.setup().roots[0].root;
    let screen = &conn.setup().roots[screen_num];

    let mode = GrabMode::ASYNC;
    let mask = EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::BUTTON_MOTION;
    let mask = u16::try_from(u32::from(mask)).unwrap();

    conn.grab_button(
        false,
        root,
        mask,
        mode,
        mode,
        x11rb::NONE,
        x11rb::NONE,
        ButtonIndex::M1,
        KeyButMask::MOD4,
    )?;

    conn.grab_button(
        false,
        root,
        mask,
        mode,
        mode,
        x11rb::NONE,
        x11rb::NONE,
        ButtonIndex::M3,
        KeyButMask::MOD4,
    )?;

    conn.grab_key(false, root, KeyButMask::MOD4, 36, mode, mode)?;

    conn.flush()?;

    let mut window_attributes: Option<ButtonPressEvent> = None;
    let mut ge = None;

    loop {
        let event = conn.wait_for_event()?;
        match event {
            KeyPress(event) => match event.detail {
                36 => {
                    match Command::new("kitty").spawn() {
                        Err(e) => println!("An error occured {:?}", e.to_string()),
                        _ => {}
                    };
                }
                _ => {
                    println!("Unhandled keycode {:?}", event.response_type)
                }
            },
            ButtonPress(event) => {
                if event.child != 0 {
                    window_attributes = Some(event.clone());
                    ge = Some(get_geometry(&conn, event.child).unwrap().reply().unwrap());
                    // println!("{:?}", get_geometry(&conn, event.child).unwrap().reply());
                }
            }
            MotionNotify(event) => {
                if let Some(window_attributes) = window_attributes {
                    if let Some(ge) = ge {
                        if window_attributes.child == 0 {
                            continue;
                        }

                        let xdiff: i16 = (event.root_x - window_attributes.root_x).into();
                        let ydiff: i16 = (event.root_y - window_attributes.root_y).into();

                        let new_width = if window_attributes.detail == 3 {
                            xdiff.try_into().unwrap()
                        } else {
                            0
                        };
                        let new_height = if window_attributes.detail == 3 {
                            ydiff.try_into().unwrap()
                        } else {
                            0
                        };

                        println!("{:?}", new_width);
                        println!("{:?}", new_height);

                        let new_location = ConfigureWindowAux {
                            x: Some(
                                (ge.x
                                    + if window_attributes.detail == 1 {
                                        xdiff
                                    } else {
                                        0
                                    })
                                .into(),
                            ),
                            y: Some(
                                (ge.y
                                    + if window_attributes.detail == 1 {
                                        ydiff
                                    } else {
                                        0
                                    })
                                .into(),
                            ),
                            width: Some(max(1, i32::from(ge.width) + new_width).try_into().unwrap()),
                            height: Some(max(1, i32::from(ge.height) + new_height).try_into().unwrap()),
                            border_width: None,
                            sibling: None,
                            stack_mode: None,
                        };
                        // println!("{:?}", new_location);
                        // println!("{:?}", event);
                        // println!("{:?}", window_attributes);
                        configure_window(&conn, event.child, &new_location)?;
                        conn.flush()?;
                    }
                }
            }
            ClientMessage(event) => {
                println!("Client Message Event {:?}", event);
            }
            _ => {
                // println!("Event {:?} not implemented yet", event)
            }
        }
    }
}
