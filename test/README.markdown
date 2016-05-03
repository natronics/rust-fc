Send Test Data To rust-fc
=========================

The design of the PSAS flight computer has a central process that is updated with current sensor and state data via sockets. In order to send some data to the flight computer without having to fly to Portland, visit the rocket lab, and run your copy of the code on the actual rocket, this python script will generate valid rocket data and send it to localhost on the correct ports.

It requires the `psas_packet` python library written by PSAS, which contains all the message structure definitions and helper functions for sending over the network.


Installing
----------

**With virtualenv**

Best practice in python is to put each project you work on in a "virtual environment" so there are never any library version conflicts. Assuming you have the popular `virtualenvwrapper` package installed, create a new virtualenv, in this case named `rust-fc-test`, but you can pick your own name.

    $ mkvirtualenv rust-fc-test

Then install any dependencies:

    $ pip install -r requirements.txt

When you're finished and want to do something else with your life, leave the virtualenv:

    $ deactivate

And when you want to use it again

    $ workon rust-fc-test

**Without virtualenv**

Or you can install to your user account:

    $ pip install --user -r requirements.txt

Failing that, install globally _not recommended_:

    $ sudo pip install -r requirements.txt


Running
-------

Once you have all the dependencies installed then all you have to do is run the script:

    $ ./send_data.py
