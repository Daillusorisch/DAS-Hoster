import struct
import random

def generate_data_packet(packet_number):
    packet = bytearray([packet_number])
    
    # 生成10,000个12位数据点并两两合并
    for _ in range(5000): 
        # 随机生成两个12位数据点
        data1 = random.randint(0, 4095)
        data2 = random.randint(0, 4095)
        combined_data = (data1 << 12) | data2
        
        # 将24位数据拆分为三个字节并添加到数据包
        packet += struct.pack('>I', combined_data)[1:]  # 使用大端序 去除多余一位

    return packet

def main():
    # 设置数据包的数量
    number_of_packets = 1
    for i in range(number_of_packets):
        packet_number = (i % 255) + 1  # 控制信息从1开始，递增到255后重新开始
        packet = generate_data_packet(packet_number)
        
        #print(binascii.hexlify(packet))# 检视数据包
        
        # 将数据包保存到文件
        with open(f"packet_{packet_number}.bin", "wb") as file:
            file.write(packet)
        print(f"Generated packet {packet_number} with size {len(packet)} bytes.")

if __name__ == "__main__":
    main()
