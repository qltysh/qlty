#!/bin/bash

if [[ $# != 3 ]]; then
	username=$1
	sudo su - ${username} -c whoami
	exit 1
fi
