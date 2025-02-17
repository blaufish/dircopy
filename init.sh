#!/bin/bash

depends_on() {
	if which -- "$1" 2>&1 > /dev/null
	then
		echo -n ""
	else
		>&2 echo "please install $1, maybe sudo apt install $1"
		exit 2
	fi
}

if [[ ! -d .git ]]
then
	git init .
fi

depends_on rustc 
depends_on cargo

if [[ ! -d hello_world ]]
then
	cargo init hello_world
fi

if [[ ! -d gentestfile ]]
then
	cargo init gentestfile
fi
