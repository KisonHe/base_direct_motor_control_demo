New protocol diff:

1. Now, the motor id sequence has changed. Now all motors has CCW as positive direction, and top left motor is 1, bottom motor is 2, top right motor is 3.
2. Now, padding bytes for SpeedCurrent mixed control must be 0x00 instead of old 0x55.
