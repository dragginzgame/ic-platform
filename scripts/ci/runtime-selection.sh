#!/usr/bin/env bash
# One inert selection for runtime graph/build/installed qualification.
# shellcheck disable=SC2034 # Selection fields are consumed by the sourcing runner.
runtime_selection() {
    qualification=owner selection_mode=source transport_mode=raw adapter_mode=external
    published='' mode=run
    local seen=' ' option explicit_adapters=false selected_mode=false
    while [[ $# -gt 0 ]]; do
        option="$1"; shift
        case "$seen" in *" $option "*) echo "Duplicate runtime option: $option" >&2; return 2 ;; esac
        seen+="$option "
        case "$option" in
            --timers) qualification=timers ;;
            --external-adapters) explicit_adapters=true ;;
            --prepared-call) transport_mode=prepared ;;
            --published)
                [[ $# -gt 0 && "$1" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || {
                    echo 'Expected an exact published runtime version' >&2; return 2;
                }
                published="$1"; shift; selection_mode=published ;;
            run|--check-graph|--build-only|--prepare|--refresh-lock)
                [[ "$selected_mode" == false ]] || { echo 'Select one runtime operation' >&2; return 2; }
                mode="$option"; selected_mode=true ;;
            *) echo "Unknown runtime option: $option" >&2; return 2 ;;
        esac
    done
    if [[ "$selection_mode" == published && "$explicit_adapters" == false ]]; then adapter_mode=raw; fi
    case "$selection_mode/$transport_mode/$mode" in
        published/prepared/*) echo 'The prepared-call fixture selects local sources; omit --published' >&2; return 2 ;;
        published/*/--prepare) echo 'Prepare published caches explicitly before qualification' >&2; return 2 ;;
        source/raw/--refresh-lock) ;;
        */*/--refresh-lock) echo 'Published or prepared-call qualification cannot rewrite the maintained local lock' >&2; return 2 ;;
    esac
}
