#!/bin/bash

N_PLAYERS=$1
THRESHOLD=$2  # Require 2*THRESHOLD < N_PLAYERS

echo 3
echo RootCA

echo $N_PLAYERS
echo 172.31.47.222
echo Player0.crt
echo 172.31.47.222
echo Player1.crt
echo 172.31.47.163
echo Player2.crt
echo 172.31.47.163
echo Player3.crt
#for (( i = 0; i < $N_PLAYERS; i++ ))
#do
#  echo 127.0.0.1
#  echo Player$i.crt
#done
#echo 172.31.47.222
#echo Player$((N_PLAYERS-1)).crt

echo N
echo N
echo 2
echo 300424569129657234489620267994584186881

echo $THRESHOLD
