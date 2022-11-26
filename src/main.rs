use std::cmp::max;
use std::process::Command;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event::{
    ButtonPress, ClientMessage, ConfigureNotify, ConfigureRequest, CreateNotify, DestroyNotify,
    KeyPress, MotionNotify,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;
    let screen = &conn.setup().roots[screen_num];

    println!("root {:?}", root);

    let new_window_attributes = ChangeWindowAttributesAux::new()
        .event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::BUTTON_PRESS
                | EventMask::POINTER_MOTION
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE,
        )
        .background_pixel(screen.white_pixel)
        .border_pixel(screen.black_pixel);

    conn.change_window_attributes(root, &new_window_attributes)?;

    let mode = GrabMode::ASYNC;
    let mask = EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::BUTTON_MOTION;

    conn.grab_button(
        false,
        root,
        mask,
        mode,
        mode,
        x11rb::NONE,
        x11rb::NONE,
        ButtonIndex::M1,
        x11rb::protocol::xproto::ModMask::M4,
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
        x11rb::protocol::xproto::ModMask::M4,
    )?;

    conn.grab_key(
        false,
        root,
        x11rb::protocol::xproto::ModMask::M4,
        36,
        mode,
        mode,
    )?;

    conn.flush()?;

    let mut window_attributes: Option<ButtonPressEvent> = None;
    let mut ge = None;

    loop {
        let event = conn.wait_for_event()?;
        match event {
            KeyPress(event) => match event.detail {
                36 => {
                    if let Err(e) = Command::new("kitty").spawn() {
                        println!("An error occurred {:?}", e.to_string())
                    };
                }
                _ => {
                    println!("Unhandled keycode {:?}", event.response_type)
                }
            },
            ButtonPress(event) => {
                if event.child != 0 {
                    window_attributes = Some(event);
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

                        let xdiff: i16 = event.root_x - window_attributes.root_x;
                        let ydiff: i16 = event.root_y - window_attributes.root_y;

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

                        let new_location = ConfigureWindowAux {
                            x: Some(
                                (ge.x
                                    + if window_attributes.detail == 1 {
                                        event.root_x - window_attributes.root_x
                                    } else {
                                        0
                                    })
                                .into(),
                            ),
                            y: Some(
                                (ge.y
                                    + if window_attributes.detail == 1 {
                                        event.root_y - window_attributes.root_y
                                    } else {
                                        0
                                    })
                                .into(),
                            ),
                            width: Some(
                                max(1, i32::from(ge.width) + new_width).try_into().unwrap(),
                            ),
                            height: Some(
                                max(1, i32::from(ge.height) + new_height)
                                    .try_into()
                                    .unwrap(),
                            ),
                            border_width: None,
                            sibling: None,
                            stack_mode: None,
                        };
                        configure_window(&conn, event.child, &new_location)?;
                        conn.flush()?;
                    }
                }
            }
            ClientMessage(event) => {
                println!("Client Message Event {:?}", event);
            }
            ConfigureNotify(event) => {
                println!("Configure Notify Event {:?}", event);

            }
            ConfigureRequest(event) => {
                println!("Configure Request Event {:?}", event);
            }
            _ => {
                // println!("Event {:?} not implemented yet", event)
            }
        }
    }
}
