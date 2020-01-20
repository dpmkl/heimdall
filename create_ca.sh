#!/bin/bash

# Creates a simple self signed CA for testing, valid 10 years
openssl req -x509 -newkey rsa:4096 -keyout privkey.pem -out fullchain.pem -days 3650 -nodes

