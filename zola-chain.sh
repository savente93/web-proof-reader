#!/bin/bash

zola build;
if [ "$?" -ne 0 ]
   then
	exit 1
   fi
git add -u public --renormalize #add the generated file to staging area to keep pre-commit happy
proof-reader
