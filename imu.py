#!/usr/bin/env python3
# -*- coding:utf-8 -*-

import os
import sys
import time
import can
import struct

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from utility import CanTrans
from utility.imu_can_msgs import HexImuMessage,HEXCANIDFunctionImu

# set IMU_MODEL and IMU_NUMBER to None if you want to auto capture one
IMU_MODEL = 0x02
IMU_NUMBER = 0x01

class Imu:

    def __init__(self):
        self.can_trans = CanTrans('hexcan0')
        self.angle = [0, 0, 0]          #deg
        self.angle_speed = [0, 0, 0]    #deg/s
        self.angle_acc = [0, 0, 0]      #m/s^2
        self.quaternion = [0, 0, 0, 0]  #w,x,y,z
        
    def try_decode_data(self, can_msg: HexImuMessage):
        if can_msg.func == HEXCANIDFunctionImu.ANGLE_REPORT.value:
            data = struct.unpack('<hhh', can_msg.data)
            self.angle = [data[0]/100.0, data[1]/100.0, data[2]/100.0]
        elif can_msg.func == HEXCANIDFunctionImu.ANGLE_SPEED_REPORT.value:
            data = struct.unpack('<hhh', can_msg.data)
            self.angle_speed = [data[0]/100.0, data[1]/100.0, data[2]/100.0]
        elif can_msg.func == HEXCANIDFunctionImu.ANGLE_ACC_REPORT.value:
            data = struct.unpack('<hhh', can_msg.data)
            self.angle_acc = [data[0]/100.0, data[1]/100.0, data[2]/100.0]
        elif can_msg.func == HEXCANIDFunctionImu.QUATERNION_REPORT1.value:
            data = struct.unpack('<ff', can_msg.data)
            self.quaternion[0] = data[0]
            self.quaternion[1] = data[1]
        elif can_msg.func == HEXCANIDFunctionImu.QUATERNION_REPORT2.value:
            data = struct.unpack('<ff', can_msg.data)
            self.quaternion[2] = data[0]
            self.quaternion[3] = data[1]
    

    def run(self):
        # enable IMU
        if IMU_MODEL == None or IMU_NUMBER == None:
            print("Heve not set IMU_MODEL or IMU_NUMBER, will auto capture one")
            (model, number) = self.can_trans.capture_model()
            self.can_trans.enable_imu(model, number)
        else:
            self.can_trans.enable_imu(IMU_MODEL, IMU_NUMBER)

        # start receive imu data
        while True:
            can_msg: HexImuMessage = self.can_trans.try_recv_msg()
            self.try_decode_data(can_msg)
            print(f"Angle: {self.angle}, Angle Speed: {self.angle_speed}, Angle Acc: {self.angle_acc}, Quaternion: {self.quaternion}")


if __name__ == '__main__':
    imu = Imu()
    imu.run()
