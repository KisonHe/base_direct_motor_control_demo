#![allow(dead_code)]

use std::array::from_mut;

use colored::Colorize;
use futures_util::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use socketcan::tokio::AsyncCanSocket;
use socketcan::{CanDataFrame, CanFilter, CanSocket, EmbeddedFrame, Frame, SocketOptions};
use tokio::sync::Mutex;

// lazy_static! {
//     static ref VEHICLE: Mutex<chassis::ChassisInterface> = Mutex::new(Default::default());
// }

static ID_CLASS_MOTOR: u8 = 0x05;
static ID_TYPE_MOTOR: u8 = 0x05;

struct Motor {
    id_number: u8,
    enabled: bool,
    target_speed: u32,
}

// There should be a lock locking read and write, but this is just a demo
impl Motor {
    fn new(id_number: u8) -> Self {
        Self {
            id_number,
            enabled: false,
            target_speed: 0,
        }
    }
    async fn periodic_update(&self) {}
    fn handle_frame(&mut self, frame: &CanDataFrame) {
        let mask = ((ID_CLASS_MOTOR as u32) << 24)
            | (ID_TYPE_MOTOR as u32) << 16
            | (self.id_number as u32) << 8;
        if frame.raw_id() & 0xFFFFFF00 == mask {
            // This frame is for this motor.
            // You can decode according to the frame data.
            // Now we ony decode heartbeat
            let id_function = (frame.raw_id() & 0xFF) as u8;
            // let  = frame.into()
            if id_function == 0xB0 {
                // Heartbeat
                self.enabled = frame.data()[0] == 1;
            }
        }
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[tokio::main]
async fn main() {
    let can_name = "can0".to_string();
    let mut bus = socketcan::tokio::CanSocket::open(&can_name).unwrap();
    let (tx, mut rx) = bus.split();

    let mut motor1 = Motor::new(1);
    let mut motor2 = Motor::new(2);
    let mut motor3 = Motor::new(3);

    tokio::spawn(async move {
        let mut rx = rx;
        loop {
            if let Some(Ok(socketcan::CanFrame::Data(frame))) = rx.next().await {
                motor1.handle_frame(&frame);
                motor2.handle_frame(&frame);
                motor3.handle_frame(&frame);
            }
        }
    });

    loop {
        // Main loop does nothing.
        std::future::pending::<i32>().await;
    }
}
