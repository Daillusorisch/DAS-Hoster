# import numpy as np
# from scipy.signal import fftconvolve
# from scipy.fftpack import fft, ifft

# h = np.array([1.0, 2.0, 3.0])

# # Convolution kernel (h)
# a = np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 20.0, 50.0])

# # Convolution result (g)
# g = fftconvolve(a, h)
# print(f"{g = }")

# # Pad h to match the length of g
# h_padded = np.pad(h, (0, len(g) - len(h)), mode='constant')
# # print(h_padded)

# # Fourier transforms of h and g
# H = fft(h_padded)
# G = fft(g)
# A = fft(a)

# print(f"{H = }")
# print(f"{G = }")
# print(f"{ifft(G) = }")
# print(f"{A = }")

# # Deconvolution in the Fourier domain
# F = G / H

# # Inverse Fourier transform to obtain the original sequence (f)
# f = ifft(F).real

# print("Original sequence (f):", f)

import numpy as np
import matplotlib.pyplot as plt

# 生成输入复合信号
fs = 1000  # 采样频率
t = np.arange(0, 1, 1/fs)  # 时间
freqs = [5, 30, 60, 100]  # 频率
sig = np.sin(2*np.pi*freqs[0]*t) + np.sin(2*np.pi*freqs[1]*t) + np.sin(2*np.pi*freqs[2]*t) + np.sin(2*np.pi*freqs[3]*t)

# 对输入信号进行FFT
f, F_sig = np.fft.fftfreq(len(sig), 1/fs), np.fft.fft(sig)

# 设置低通滤波器的截止频率（60Hz）
fc = 60

# 计算低通滤波器的频域衰减因子（0.5为阈值）
Wc = 0.5 * np.sinc(fc / (fs / 2))

# 对FFT结果进行低通滤波
F_filtered = F_sig * np.where(np.abs(f) < fc, 1, Wc)

# 进行反FFT得到滤波后的信号
sig_filtered = np.fft.ifft(F_filtered)

# 绘制原始信号和滤波后的信号
plt.figure(figsize=(10, 5))
plt.plot(t, sig, label='orgin')
plt.plot(t, np.real(sig_filtered), label='pass')
plt.xlabel('time')
plt.ylabel('abs')
plt.title('ir')
plt.legend()
plt.show()