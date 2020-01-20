#!/bin/bash 

if [ -d ".rpm" ]; then
  rm -rf .rpm 
fi

cargo rpm init --service dist/heimdall.service

cargo rpm build



