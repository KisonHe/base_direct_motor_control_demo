#![allow(dead_code)]

use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use socketcan::CanDataFrame;
use socketcan::{EmbeddedFrame, ExtendedId, Frame};
use std::time::Duration;
use tokio::sync::Mutex;

use socketcan::CanFrame;

lazy_static! {
    static ref MOTOR1: Mutex<Motor> = Mutex::new(Motor::new(1));
    static ref MOTOR2: Mutex<Motor> = Mutex::new(Motor::new(2));
    static ref MOTOR3: Mutex<Motor> = Mutex::new(Motor::new(3));
}

static ID_CLASS_MOTOR: u8 = 0x05;
static ID_TYPE_MOTOR: u8 = 0x05;

static ID_FUNCTION_MODE6_SET_PARAM: u8 = 0x06;
static ID_FUNCTION_SET_DRIVER_MODE: u8 = 0x01;

static TARGET_DRIVER_MODE: u8 = 6;

// todo correct this value to real vehicle radius
static VEHICLE_RADIUS: f32 = 0.3274;
static VFACTOR: f32 = 0.8660254037844386f32; // sqrtf32(3) / 2.0f32;
static VFACTOR1: f32 = 0.5f32;
static VFACTOR2: f32 = 0.5773502691896257f32; // sqrtf32(3) / 3.0f32;
static VFACTOR3: f32 = 0.3333333333333333f32;
static VFACTOR4: f32 = 0.6666666666666666f32;
// todo correct this value to real value
static REDUCTION_RATIO: f32 = 1.0f32;
static WHEEL_RADIUS: f32 = 0.103f32;

fn base_spd_2_motors_rpm(x: f32, y: f32, z: f32) -> (i16, i16, i16) {
    let ms2rpm_magic = REDUCTION_RATIO * 60.0f32 / (std::f32::consts::PI * WHEEL_RADIUS * 2.0f32);
    // println!("ms2rpm_magic: {}", ms2rpm_magic);
    let motor1_ms = -y + z * VEHICLE_RADIUS;
    // println!("motor1_ms: {}", motor1_ms);
    let motor1_rpm = (motor1_ms * ms2rpm_magic) as i16;
    // println!("motor1_rpm: {}", motor1_rpm);

    let motor2_ms = y * VFACTOR1 + z * VEHICLE_RADIUS - x * VFACTOR;
    let motor2_rpm = (motor2_ms * ms2rpm_magic) as i16;

    let motor3_ms = -y * VFACTOR1 - z * VEHICLE_RADIUS - x * VFACTOR;
    let motor3_rpm = (motor3_ms * ms2rpm_magic) as i16;

    println!(
        "motor1_ms: {}, motor2_ms: {}, motor3_ms: {}",
        motor1_ms, motor2_ms, motor3_ms
    );

    return (motor1_rpm, motor2_rpm, motor3_rpm);
}

struct Motor {
    pub id_number: u8,
    pub read_speed: f32,
}

// There should be a lock locking read and write, but this is just a demo
impl Motor {
    fn new(id_number: u8) -> Self {
        Self {
            id_number,
            // should be dealt in handle, however this is just a FK demo
            read_speed: 0f32,
        }
    }
}

async fn rotate_motor<T>(motor: &Motor, tx: &mut T, rpm: i16)
where
    T: futures_util::sink::Sink<CanFrame, Error = socketcan::Error> + Unpin,
{
    // Set driver mode
    let id = ((ID_CLASS_MOTOR as u32) << 24)
        | (ID_TYPE_MOTOR as u32) << 16
        | (motor.id_number as u32) << 8
        | ID_FUNCTION_SET_DRIVER_MODE as u32;
    let frame = CanDataFrame::new(
        ExtendedId::new(id).unwrap(),
        &[TARGET_DRIVER_MODE, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55], // 0x55 is just a placeholder. Filling data to 8 bytes.
    )
    .unwrap();
    tx.send(socketcan::CanFrame::Data(frame)).await.unwrap();
    let id = ((ID_CLASS_MOTOR as u32) << 24)
        | (ID_TYPE_MOTOR as u32) << 16
        | (motor.id_number as u32) << 8
        | ID_FUNCTION_MODE6_SET_PARAM as u32;
    let frame = CanDataFrame::new(
        ExtendedId::new(id).unwrap(),
        &[
            // 0x05 Class is special. Different from the rest, it has no Standard Instruction Set, and data is in big endian form.
            (4000u16 >> 8) as u8,
            (4000u16 & 0xFF) as u8,
            (rpm >> 8) as u8,
            (rpm & 0xFF) as u8,
            0x55,
            0x55,
            0x55,
            0x55,
        ], // 0x55 is just a placeholder. Filling data to 8 bytes.
    )
    .unwrap();
    tx.send(socketcan::CanFrame::Data(frame)).await.unwrap();
}

fn generate_reboot_command(id: u8) -> CanFrame {
    let id =
        ((ID_CLASS_MOTOR as u32) << 24) | (ID_TYPE_MOTOR as u32) << 16 | (id as u32) << 8 | 0x00;
    socketcan::CanFrame::Data(
        CanDataFrame::new(
            ExtendedId::new(id).unwrap(),
            &[0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55], // 0x55 is just a placeholder. Filling data to 8 bytes.
        )
        .unwrap(),
    )
}

#[tokio::main]
async fn main() {
    let can_name = "xvcan0".to_string();
    let mut bus = socketcan::tokio::CanSocket::open(&can_name).unwrap();
    let (mut tx, mut rx) = bus.split();
    // First, send reboot commands
    tokio::time::sleep(Duration::from_millis(100)).await;
    tx.send(generate_reboot_command(0x01)).await.unwrap();
    tx.send(generate_reboot_command(0x02)).await.unwrap();
    tx.send(generate_reboot_command(0x03)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    tokio::spawn(async move {
        let mut rx = rx;
        loop {
            if let Some(Ok(socketcan::CanFrame::Data(frame))) = rx.next().await {
                let mask = ((ID_CLASS_MOTOR as u32) << 24) | (ID_TYPE_MOTOR as u32) << 16;
                if frame.raw_id() & 0xFFFF0000 == mask {
                    let id = (frame.raw_id() & 0x0000FF00) as u8;
                    if id == 1 {
                        let mut motor1 = MOTOR1.lock().await;
                        // handle frame here, B1 and B2 id functions
                        // motor1.handle_frame(&frame);
                    } else if id == 2 {
                        let mut motor2 = MOTOR2.lock().await;
                        // motor2.handle_frame(&frame);
                    } else if id == 3 {
                        let mut motor3 = MOTOR3.lock().await;
                        // motor3.handle_frame(&frame);
                    }
                }
            }
        }
    });

    loop {
        tokio::time::sleep(Duration::from_millis(5)).await;
        // Change this line to make the robot move at desired speed.
        // Unit is m/s. DO NOT SET A LARGE SPEED OR YOU WILL RISK CRASHING STUFF.
        let (x, y, z) = (0.3f32, 0.0f32, 0.0f32);
        // println!("x: {}, y: {}, z: {}", x, y, z);
        let (spd1, spd2, spd3) = base_spd_2_motors_rpm(x, y, z);
        println!("FK: spd1: {}, spd2: {}, spd3: {}", spd1, spd2, spd3);
        {
            // lock is not necessary, but lets show how to use it
            let mut motor1 = MOTOR1.lock().await;
            rotate_motor(&motor1, &mut tx, spd1).await;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
        {
            let mut motor2 = MOTOR2.lock().await;
            rotate_motor(&motor2, &mut tx, spd2).await;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
        {
            let mut motor3 = MOTOR3.lock().await;
            rotate_motor(&motor3, &mut tx, spd3).await;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
