#!/usr/bin/env python2.7

# A Python script for running a Stencila session in a Docker container

import time
import stencila

stencila.serve()

while True: time.sleep(1)
