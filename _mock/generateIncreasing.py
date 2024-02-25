import struct
import binascii

def generate_increasing_decreasing_data(total_points=10000):
    # 初始化数据列表
    data_points = []
    max_value = 4095
    # 生成递增至4095然后递减至0的数据序列
    increasing = list(range(0, max_value + 1))
    decreasing = list(range(max_value, -1, -1))
    sequence = increasing + decreasing
    # 重复序列

    while len(data_points) < total_points:
        data_points.extend(sequence)
    
    # 超出截断
    data_points = data_points[:total_points]
    return data_points

def combine_data_points(data_points):
    # 将数据点两两合并为3字节
    combined_bytes = bytearray()
    for i in range(0, len(data_points), 2):
        data1 = data_points[i]
        data2 = data_points[i+1] if i+1 < len(data_points) else 0
        combined_data = (data1 << 12) | data2
        combined_bytes += struct.pack('>I', combined_data)[1:]
    return combined_bytes

def generate_data_packet(packet_number, data_points):
    # 初始化数据包，先添加控制信息
    packet = bytearray([packet_number])
    # 添加合并后的数据点
    packet += combine_data_points(data_points)
    return packet

def main():
    packet_number = 1
    data_points = generate_increasing_decreasing_data()
    packet = generate_data_packet(packet_number, data_points)

    # print(binascii.hexlify(packet)) # 检视数据包
    
    # 将数据包保存到文件
    with open(f"./data/packet_{packet_number}.bin", "wb") as file:
        file.write(packet)
    print(f"Generated packet {packet_number} with size {len(packet)} bytes.")

if __name__ == "__main__":
    main()
