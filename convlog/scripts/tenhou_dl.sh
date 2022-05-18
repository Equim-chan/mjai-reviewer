#!/usr/bin/env bash

LOGID="$1"
curl -SsL --compressed -H "Referer: https://tenhou.net/" "https://tenhou.net/5/mjlog2json.cgi?$LOGID"
