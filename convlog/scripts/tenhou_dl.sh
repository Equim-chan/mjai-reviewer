#!/bin/bash

LOGID="$1"
curl -SsL --compressed -H "Referer: https://tenhou.net/6/?log=$LOGID" "https://tenhou.net/5/mjlog2json.cgi?$LOGID"
