#!/bin/sh
set -eu
export DISPLAY=:0
openbox &
wm=$!
shell=
cleanup() {
    if [ -n "$shell" ]; then kill "$shell" 2>/dev/null || :; fi
    kill "$wm" 2>/dev/null || :
}
trap cleanup EXIT
trap 'exit 1' INT TERM
/usr/local/bin/aera-shell &
shell=$!
i=0
while [ "$i" -lt 90 ]; do
    kill -0 "$shell" || exit 1
    if xdotool search --onlyvisible --name '^AERA Shell$' >/dev/null 2>&1; then
        echo 'AERA_GUI_READY'
        break
    fi
    i=$((i+1))
    sleep 1
done
[ "$i" -lt 90 ] || exit 1
wait "$shell"
