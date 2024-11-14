import can
import struct
import time
from typing import Tuple
from .imu_can_msgs import HEXCANIDFunctionImu,HEXCANIDClass
from .imu_can_msgs import HexImuMessage,InvalidCANIDError,HEXCANIDFunctionStd


class CanTrans:
    def __init__(self, channel='hexcan0', bustype='socketcan', bitrate=500000):
        self.bus = can.interface.Bus(channel=channel, bustype=bustype, bitrate=bitrate)
        print("\033[32m" + f"connect to {channel} with bitrate {bitrate} success!" + "\033[0m")

    def try_recv_msg(self) -> HexImuMessage:
        while True:
            try:
                if not self.bus:
                    print("CAN bus is not initialized.")
                    return None

                # wait for message
                msg = self.bus.recv(0.1)

                # decode message
                if msg:
                    can_id = msg.arbitration_id
                    try:
                        hex_msg = HexImuMessage(can_id, msg.data)
                        return hex_msg
                    except InvalidCANIDError as e:
                        # print(f"Error: {e}")
                        pass

            except KeyboardInterrupt:
                print("receiver keyboard interrupt, exit")
                exit()
                break

            except Exception as e:
                print("\033[31mFailed to receive CAN message: {e}\033[0m")
                break


    def capture_model(self) -> Tuple[int, int]:
        start_time = time.time()
        while True:
            try:
                if not self.bus:
                    print("CAN bus is not initialized.")
                    continue

                msg = self.bus.recv(1)
                can_time = time.time()
                if msg:
                    can_id = msg.arbitration_id
                    try:
                        hex_msg = HexImuMessage(can_id, msg.data, can_time)
                        print("Capture model and number:", hex_msg.model, hex_msg.number)
                        return hex_msg.model, hex_msg.number
                    except InvalidCANIDError:
                        continue
            except KeyboardInterrupt:
                print("Capture keyboard interrupt, exit")
                exit()

            except Exception as e:
                print(f"\033[31mFailed to open CAN: {e}\033[0m")
                break


    def enable_imu(self, model:int, number:int):
        can_id = ((HEXCANIDClass.IMU.value) << 24) | ((model) << 16) | ((number) << 8) | (HEXCANIDFunctionStd.GENERAL_SETTING.value)
        data = [HEXCANIDClass.IMU.value, model, number, 0x01]
        self.send_msg(can_id, data)

    # # template: self.can_trans.set_imu_fb_period(IMU_MODEL, IMU_NUMBER, HEXCANIDFunctionImu.ANGLE_SET, 0)
    # def set_imu_fb_period(self, model:int, number:int, func:HEXCANIDFunctionImu, period:int):
    #     can_id = ((HEXCANIDClass.IMU.value) << 24) | ((model) << 16) | ((number) << 8) | (func.value)
    #     if period == 0:
    #         data = 0x00
    #     elif period > 255:
    #         data = 0xff
    #     elif period < 10:
    #         data = 0x0a
    #     data_bytes = bytes([data])
    #     self.send_msg(can_id, data_bytes)


    def send_msg(self, id:int, data):
        message = can.Message(
            arbitration_id=id,
            data=data,
            is_extended_id=True,
        )
        try:
            self.bus.send(message)
        except Exception as e:
            print(f"Failed to send message: {e}")


    def __del__(self):
        if self.bus:
            self.bus.shutdown()
            print("CAN bus has been shut down.")
