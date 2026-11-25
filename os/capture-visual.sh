#!/usr/bin/env bash
set -euo pipefail
mkdir -p visual-evidence
export WGPU_BACKEND=vulkan
export LIBGL_ALWAYS_SOFTWARE=1
pid=
cleanup() {
    if [[ -n "$pid" ]]; then kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; fi
}
trap cleanup EXIT
capture() {
    local mode=$1 id=
    if [[ "$mode" == fallback ]]; then
        AERA_GLASS_OFF=1 target/release/aera-shell >"visual-evidence/$mode.log" 2>&1 &
    else
        target/release/aera-shell >"visual-evidence/$mode.log" 2>&1 &
    fi
    pid=$!
    for ((i=0;i<60;i++)); do
        kill -0 "$pid"
        id=$(xdotool search --onlyvisible --pid "$pid" --name '^AERA Shell$' | head -1 || :)
        [[ -z "$id" ]] || break
        sleep 1
    done
    [[ -n "$id" ]]
    sleep 4
    kill -0 "$pid"
    import -window "$id" "visual-evidence/$mode.png"
    # A blank or near-uniform frame is not acceptable evidence.
    colors=$(identify -format '%k' "visual-evidence/$mode.png")
    [[ "$colors" -gt 32 ]]
    identify -format '%wx%h\n' "visual-evidence/$mode.png" >> visual-evidence/provenance.txt
    cleanup
    pid=
}
printf 'commit=%s\nplatform=%s\nbackend_requested=Vulkan\nrender_environment=Linux Xvfb Mesa software Vulkan\ntheme=dark\nquality=0.65 (glass)\n' "$(git rev-parse HEAD)" "$(uname -sm)" > visual-evidence/provenance.txt
capture glass
capture fallback
cmp -s visual-evidence/glass.png visual-evidence/fallback.png && exit 1
exit 0
