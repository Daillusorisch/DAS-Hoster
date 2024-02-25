import struct
import binascii
import numpy as np
import scipy.signal as signal

'''
    传入频率 持续时间 采样间隔，生成对应的正弦波数据
'''
def generate_sine_wave_data(frequency, total_duration_s, sample_interval_ns=10000):
    # 正弦幅值
    amplitude = 2047
    # 生成正弦数据
    sample_interval_s = sample_interval_ns * 1e-9
    total_points = int(total_duration_s / sample_interval_s)
    times = np.arange(total_points) * sample_interval_s
    data_points = amplitude * np.sin(2 * np.pi * frequency * times)
    data_points = np.round((data_points + amplitude) * (4095 / (2 * amplitude))).astype(int)
    # 定义卷积核
    kernel = np.array([1, 1, 1, 1, 1, 1])
    # 进行卷积
    data_points = signal.convolve(data_points, kernel)
    return data_points.tolist()

'''
    将数据点两两合并为3字节
'''
def combine_data_points(data_points):
    combined_bytes = bytearray()
    for i in range(0, len(data_points), 2):
        data1 = data_points[i]
        data2 = data_points[i+1] if i+1 < len(data_points) else 0
        combined_data = (data1 << 12) | data2
        combined_bytes += struct.pack('>I', combined_data)[1:]
    return combined_bytes

'''
    生成数据包
'''
def generate_data_packet(packet_number, data_points):
    packet = bytearray([packet_number % 256])+bytearray(12)+bytearray(32)  # 控制信息从1开始，递增到255后循环
    packet += combine_data_points(data_points)
    return packet

def generate_and_save_packets(number_of_packets, total_duration_s):
    frequency = 50  # 信号频率
    sin_wave = generate_sine_wave_data(frequency, total_duration_s)
    for packet_number in range(number_of_packets):
        segment_duration_s = total_duration_s / number_of_packets
        start_s = (packet_number - 1) * segment_duration_s
        end_s = start_s + segment_duration_s
        data_points = sin_wave[int(start_s*100_000):int(end_s*100_000)]
        # print(data_points)
        packet = generate_data_packet(packet_number, data_points)
        file_name = f"./data/packet_{packet_number}_for_{segment_duration_s*1000}ms.bin"
        with open(file_name, "wb") as file: 
            file.write(packet)
        print(f"Generated {file_name} with size {len(packet)} bytes.")

def main(description_duration_s=1):
    number_of_packets = 10
    generate_and_save_packets(number_of_packets, description_duration_s)

if __name__ == "__main__":
    main(1)  # 填入持续时间 将以100ms分段
