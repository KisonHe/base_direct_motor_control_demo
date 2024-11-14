#!/usr/bin/env python3
# -*- coding:utf-8 -*-
from enum import Enum
import struct
from dataclasses import dataclass
import time

# ----------------- CAN ID constants -----------------
"""
This is a list of constants that are used by all devices.
"""
class HEXCANIDClass(Enum):
    IMU = 0x0B

class HEXCANIDFunctionStd(Enum):
    """
    This is a list of standard functions that are used by all devices.
    Note this list is not exhaustive, and each device may have its own functions.
    Also, this list is a full list of std functions, becuase this is just a demo.
    """
    GENERAL_SETTING = 0x03      # General settings. This sets the enable state of the device
    CLEAR_ERROR = 0x04          # Clears error, if success
    CLEAR_KINESTATE = 0x05      # Clears kinematic state, this will reset odometry, if success
    FIND_DEVICE = 0x07          # Find device, not all devices support this
    HEARTBEAT = 0xB0

class HEXCANIDFunctionImu(Enum):
    """
    This is a list of functions that are used by all Imu(id class = HEXCANIDClass.IMU).
    """
    ANGLE_SET = 0x11
    ANGLE_REPORT = 0xB1
    ANGLE_SPEED_SET = 0x12
    ANGLE_SPEED_REPORT = 0xB2
    ANGLE_ACC_SET = 0x13
    ANGLE_ACC_REPORT = 0xB3
    QUATERNION_SET = 0x14
    QUATERNION_REPORT1 = 0xB4
    QUATERNION_REPORT2 = 0xB5

# ----------------- IMU Message class -----------------
class HexImuMessage:
    """
    This is a class for IMU message.
    """
    def __init__(self, can_id, data: bytes):
        # 检查拓展帧
        if can_id > 0x7FF:
            self.class_id = (can_id >> 24) & 0xFF
            if self.class_id!= HEXCANIDClass.IMU.value:
                raise InvalidCANIDError(f"Invalid class_id: {self.class_id}. Expected HEXCANIDClass.IMU.")

            self.model = (can_id >> 16) & 0xFF
            self.number = (can_id >> 8) & 0xFF
            self.func = can_id & 0xFF
            
            self.data = data

            # print(f"\033[32mDecoded CAN ID: class={self.class_id:#04x}, model={self.model}, number={self.number}, func={self.func:#04x}\033[0m")

        else:
            raise InvalidCANIDError(f"Invalid CAN ID: {can_id}.")

    def __repr__(self):
        return f"HexImuMessage(class={self.class_id}, model={self.model}, number={self.number}, func={self.func}, data={self.data.hex()})"


# ----------------- Error Types -----------
class InvalidCANIDError(Exception):
    """Custom exception for invalid CAN ID"""
    pass
