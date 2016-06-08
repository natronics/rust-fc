#!/usr/bin/env python
import subprocess
import socket
import time
import sys
import select
from psas_packet import io, messages
import random

# data type
ADIS = messages.MESSAGES['ADIS']

# convert from stupid imperial units
FPS2M = 0.3048
LBF2N = 4.44822
LBS2KG = 0.453592

# Seed for deterministic random behavior
random.seed(0xBADA55)

# output from JSBSim goes to this socket (port specified in output.xml)
jsb_output = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
jsb_output.bind(("127.0.0.1", 5123))
jsb_output.setblocking(0)


# Socket writer for sending PSAS formated binary data to our flight computer
fc_imu_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
fc_imu_sock.bind(('', 35020))
fc_imu_sock.connect(('127.0.0.1', 36000))
fc_imu = io.Network(fc_imu_sock)


# Run JSBSim using Popen
p = subprocess.Popen([
    "JSBSim",
    "--realtime",
    "--suspend",
    "--nice",
    "--simulation-rate=1000",
    "--logdirectivefile=output_UDP.xml",
    "--logdirectivefile=output_file.xml",
    "--script=run.xml"
])


# settling time for JSBSim to initialize and start a TCP server for commands
time.sleep(0.5)

# Connect to JSBSim console with this socket (specified in run.xml: <input port="5124" />
jsb_console = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
jsb_console.connect(("127.0.0.1", 5124))

# Start simulation, but we've not ignited the engine yet. Should just sit there on the ground (but stream data)
jsb_console.send(b"resume\n")

# While loop for listening for data from JSBSim
started = False
seqn = 0
while True:
    # Press any key at the command line to launch the rocket
    if not started and len(select.select([sys.stdin], [], [], 0)[0]) > 0:

        # the JSBSim command to ignite motor
        jsb_console.send(b"set fcs/throttle-cmd-norm[0] 1.0\r\n")
        started = True  # so we only trigger this once

    # try reading data from JSBSim
    try:
        buf = jsb_output.recv(1500)

        if len(buf) > 1:
            data = buf.split(b',')
            time    = float(data[0].strip())
            weight  = float(data[1].strip()) * LBS2KG  # kg
            force_x = float(data[2].strip()) * LBF2N   # N
            force_y = float(data[3].strip()) * LBF2N   # N
            force_z = float(data[4].strip()) * LBF2N   # N
            vel     = float(data[5].strip()) * FPS2M   # m/s

            # The IMU measured acceleration is the forces / mass
            accel_x = force_x / weight
            accel_y = force_y / weight
            accel_z = force_z / weight

            # JSBSim will report 0 force/accel while hold-down is on
            # so we simulation sitting on the pad (1 G measured accel up)
            if not started:
                accel_x = 9.8

            # Add realistic noise to simulation
            # the noise seems to be proportional with velocity, with a lower bound
            noise = vel/200.0
            if noise < 0.1:
                noise = 0.1
            accel_x += random.gauss(0, noise)
            accel_y += random.gauss(0, noise)
            accel_z += random.gauss(0, noise)

            # Pack ADIS message
            data = {
                'VCC': 5.0,
                'Gyro_X': 0.0,
                'Gyro_Y': 0,
                'Gyro_Z': 1,
                'Acc_X': accel_x,
                'Acc_Y': accel_y,
                'Acc_Z': accel_z,
                'Magn_X': 0,
                'Magn_Y': 0,
                'Magn_Z': 0,
                'Temp': 20,
                'Aux_ADC': 0,
            }

            # send to FC and update seqn
            fc_imu.send_data(ADIS, seqn, data)

            # Fake some missing packets
            if seqn == 25:
                seqn = seqn + 3
            seqn = seqn + 1


    # ctrl-c to kill the sim. Clean up after ourselves
    except KeyboardInterrupt:
        jsb_console.send(b"quit\n")
        jsb_console.close()
        jsb_output.close()
        fc_imu_sock.close()
        p.kill()
        break

    except SystemExit:
        jsb_console.send(b"quit\n")
        jsb_console.close()
        jsb_output.close()
        fc_imu_sock.close()
        p.kill()
        break

    # keep on looping otherwise
    except:
        pass
