#!/usr/bin/env python
import subprocess
import socket
import time
import sys
import select
from psas_packet import io, messages

# data type
ADIS = messages.MESSAGES['ADIS']

# convert from stupid imperial units
FPS2M = 0.3048


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
    "--logdirectivefile=output.xml",
    "--script=run.xml"
])


# settling time for JSBSim to initialize and start a TCP server for commands
time.sleep(0.5)

# Connect to JSBSim console with this socket (specified in run.xml: <input port="5124" />
jsb_console = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
jsb_console.connect(("127.0.0.1", 5124))

# Start simulation, but we've not ignited the engine yet. Should just sit there on the ground (but stream data)
jsb_console.send("resume\n")


# While loop for listening for data from JSBSim
started = False
seqn = 0
while True:
    # Press any key at the command line to launch the rocket
    if not started and sys.stdin in select.select([sys.stdin], [], [], 0)[0]:

        # the JSBSim command to ignite motor
        jsb_console.send('set fcs/throttle-cmd-norm[0] 1.0\r\n')
        started = True  # so we only trigger this once

    # try reading data from JSBSim
    try:
        buf = jsb_output.recv(1500)
        if len(buf) > 1:
            data = buf.split(',')
            time  = float(data[0].strip())
            acc_U = float(data[1].strip())
            acc_V = float(data[2].strip())
            acc_W = float(data[3].strip())

            # uncomment to print data to console
            #print time, acc_U, acc_V, acc_W

            # Pack ADIS message
            data = {
                'VCC': 5.0,
                'Gyro_X': 0.0,
                'Gyro_Y': 0,
                'Gyro_Z': 1,
                'Acc_X': -acc_U * FPS2M,  # inverted to match IMU on rocket
                'Acc_Y':  acc_V * FPS2M,
                'Acc_Z':  acc_W * FPS2M,
                'Magn_X': 0,
                'Magn_Y': 0,
                'Magn_Z': 0,
                'Temp': 20,
                'Aux_ADC': 0,
            }

            # send to FC and update seqn
            fc_imu.send_data(ADIS, seqn, data)
            seqn = seqn + 1


    # ctrl-c to kill the sim. Clean up after ourselves
    except KeyboardInterrupt, SystemExit:
        jsb_console.send("quit\n")
        jsb_console.close()
        jsb_output.close()
        fc_imu_sock.close()
        p.kill()
        break

    # keep on looping otherwise
    except:
        pass
