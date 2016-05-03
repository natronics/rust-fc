#!/usr/bin/env python
from psas_packet import io, messages
from contextlib import closing
import socket
import time

# Data type we're going to use
ADIS = messages.MESSAGES['ADIS']

# Data to pack
data = {
    'VCC': 5.0,
    'Gyro_X': 0.0,
    'Gyro_Y': 0,
    'Gyro_Z': 1,
    'Acc_X': -9.8,
    'Acc_Y': 0,
    'Acc_Z': 0,
    'Magn_X': 53e-6,
    'Magn_Y': 0,
    'Magn_Z': 0,
    'Temp': 20,
    'Aux_ADC': 0,
}

# Open a UDP socket
with closing(socket.socket(socket.AF_INET, socket.SOCK_DGRAM)) as sock:
    sock.bind(('', 35020))
    sock.connect(('127.0.0.1', 36000))

    # Send to FC
    net = io.Network(sock)
    net.send_data(ADIS, 0, data)

