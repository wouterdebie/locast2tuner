#!/usr/bin/with-contenv bash

locast2tuner -b 0.0.0.0 -d 8 -r --tuner_count 4
exit $?
