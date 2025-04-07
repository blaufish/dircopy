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

rust_init() {
	local dir="$1"
	if [[ -d "$dir" ]]
	then
		return
	fi
	cargo init "$dir"
}

if [[ ! -d .git ]]
then
	git init .
fi

depends_on rustc 
depends_on cargo

rust_init hello_world
rust_init gentestfile
rust_init filecopy
rust_init dircopy
