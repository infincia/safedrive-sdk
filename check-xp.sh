#!/bin/bash


#while read api; do
#	echo ------------------------------------------------------------------------------------
#	echo Checking $api
#	echo ------------------------------------------------------------------------------------
#	grep --exclude libsecur32 --exclude libcrypt32 --exclude libktmw32 --exclude libws2_32 --exclude libadvapi32 --exclude libkernel32 --exclude sddk --exclude safedrive.exe -r "$api" $1
#	echo ------------------------------------------------------------------------------------	
#	echo Done checking $api
#done <xp.api

rg -f xp.api -F -g '*.rlib' $api

