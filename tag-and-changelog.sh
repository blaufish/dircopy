#!/bin/bash

LAST_RELEASE_TAG=""
declare -A COMMIT
declare -A PCOMMIT


#set -x
set -e

get_version() {
	fgrep -A5 '[package]' Cargo.toml | grep version | awk -F'"' '{print $2}'
}

release_tags() {
	if [[ "$1" == "-r" ]]
	then
		git tag --list | egrep '^v[0-9]+.[0-9]+.[0-9]+$' | sort -r --version-sort
	else
		git tag --list | egrep '^v[0-9]+.[0-9]+.[0-9]+$' | sort --version-sort
	fi
}

index_tags() {
	local commit
	local tag
	local p_commit=""
	for tag in $(release_tags)
	do
		LAST_RELEASE_TAG="$tag"
		commit="$(git rev-list -n 1 $tag)"
		COMMIT["$tag"]="$commit"
		PCOMMIT["$tag"]="$p_commit"
		p_commit="$commit"
	done
}

describe_version() {
	tag="$1"
	commit="$2"
	p_commit="$3"
	echo "# $tag"
	echo ""
	if [[ "$commit" == "HEAD" ]]
	then
		echo "> $DESCRIPTION"
	else
		git tag -l "$tag" --format='%(contents)' | awk '{print "> "$0}'
	fi
	if [[ "$p_commit" != "" ]]
	then
		echo ""
		echo "Files:"
		echo '``` plain'
		git diff --name-status "$p_commit..$commit"
		echo '```'
	fi
	echo ""
	echo "Commits:"
	echo '``` plain'
	if [[ "$p_commit" == "" ]]
	then
		git log --graph --oneline --root "$tag"
	else
		git log --graph --oneline "$p_commit..$commit"
	fi
	echo '```'
	if [[ "$p_commit" != "" ]]
	then
		echo ""
	fi
}

first_md() {
	local commit
	local tag
	local p_commit
	for tag in $(release_tags -r)
	do
		p_commit=${PCOMMIT["$tag"]}
		commit=${COMMIT["$tag"]}
		describe_version "$tag" "$commit" "$p_commit"
	done
}

next_version() {
	tag="$1"
	commit="HEAD"
	p_commit=${LAST_RELEASE_TAG["$LAST_RELASE_TAG"]}
	describe_version "$tag" "$commit" "$p_commit"
}

DESCRIPTION=
if [[ "$1" == "" ]]
then
    echo "Usage:"
    echo " $0 \"Description of version tag\""
    exit 1
else
	DESCRIPTION="$1"
fi

version=$(get_version)
tag=$(git tag -l -- "v$version")

git fetch --tags

if [ "$tag" != "" ]
then
    echo "Version $version allready exists."
    exit 1
fi

DESCRIPTION="Release version $version. $DESCRIPTION"

index_tags
if [[ ! -f changelog.md ]]
then
	first_md >> changelog.md
fi

if [[ "$version" != "" ]]
then
	if fgrep -q -- "# $version" changelog.md
	then
		echo "Skipping updating changelog.md"
	else
		mv changelog.md changelog.md.bak
		(
			next_version "$version"
			first_md
		) >> changelog.md
	fi
fi

./build-with-podman.sh

porcelain="$(git status --porcelain=v2)"

if [ "$porcelain" ]
then
	echo "Directory is not clean. Exiting!"
	exit 1
fi

git tag -a "v$version" -m "$DESCRIPTION"
