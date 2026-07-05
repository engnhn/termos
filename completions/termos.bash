_termos_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    opts="add connect list delete update quick-command qc"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi

    local cmd="${COMP_WORDS[1]}"
    if [[ "${cmd}" == "quick-command" || "${cmd}" == "qc" ]]; then
        if [[ ${COMP_CWORD} -eq 2 ]]; then
            COMPREPLY=( $(compgen -W "list add edit delete" -- ${cur}) )
            return 0
        elif [[ ${COMP_CWORD} -eq 3 ]]; then
            local nicknames=$(termos _list-nicknames 2>/dev/null)
            COMPREPLY=( $(compgen -W "${nicknames}" -- ${cur}) )
            return 0
        fi
    fi

    case "${prev}" in
        connect|delete)
            local nicknames=$(termos _list-nicknames 2>/dev/null)
            COMPREPLY=( $(compgen -W "${nicknames}" -- ${cur}) )
            return 0
            ;;
        *)
            ;;
    esac
}

complete -F _termos_completions termos
