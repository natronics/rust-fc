
# Comparing `rust-fc` To Simulation Output

The simulator proccessing code adds realisic noise to the IMU input before sending it to `rust-fc`.

We'll compare the clean "ideal" simulator numbers to what was actually received by `rust-fc`



boundary?


## Message Receive Time

In JSBSim the IMU messages are requested to be sent at the real IMU rate of 819.2 Hz:

    <output name="localhost" type="SOCKET" protocol="UDP" port="5123" rate="819.2">

But there they are then processed in python for noise and binary packing. Then it's sent as UDP packets which may get lost. Let's see how they appear in the flight comptuer.




![](results_files/results_3_0.png)





![](results_files/results_4_0.png)


## IMU Noisy Acceleration

Here we see the noise put into the IMU data and the true acceleration.




![](results_files/results_6_0.png)


## State Tracking

The flight comptuer only knows the Inertial state (acceleration). It keeps track of velocity and altitude by integrating this signal. Here we compare `rust-fc` internal state to the exact numbers from the simulator.






![](results_files/results_9_0.png)





![](results_files/results_10_0.png)



