#!/usr/bin/env python
import subprocess
import socket
import time
import sys
import select


# output from JSBSim goes to this socket (specified in output.xml)
jsb_output = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
jsb_output.bind(("127.0.0.1", 5123))
jsb_output.setblocking(0)


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

# Connect to JSBSim console with this socket (specified in run.xml <input port="5124" />
jsb_console = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
jsb_console.connect(("127.0.0.1", 5124))

# Start simulation, but we've not ignited the engine yet. Should just sit there on the ground (but stream data)
jsb_console.send("resume\n")


# While loop for listening for data from JSBSim
started = False
while True:
    # Press any key at the command line to launch the rocket
    if not started and sys.stdin in select.select([sys.stdin], [], [], 0)[0]:

        # the JSBSim command to ignite motor
        jsb_console.send('set %s %s\r\n' % ("fcs/throttle-cmd-norm[0]", "1.0"))
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
            print time, acc_U, acc_V, acc_W

    # ctrl-c to kill the sim. Clean up after ourselves
    except KeyboardInterrupt, SystemExit:
        jsb_console.send("quit\n")
        jsb_console.close()
        jsb_output.close()
        p.kill()
        break

    # keep on looping otherwise
    except:
        pass
