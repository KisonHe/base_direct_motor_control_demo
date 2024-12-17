# How to use

## Before you start

Make sure you have lifted your robot, for example putting it on a box. Make sure the wheels are not touching the ground.

## Get it running

1. Set up your linux socketcan.
2. Open `main.rs`
3. Change `xvcan0` to your can interface.
4. Change `let (x, y, z) = (0.3f32, 0.0f32, 0.0f32);` to speed you want.
5. Do `cargo run` and the robot should start moving. 
