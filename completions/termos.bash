_termos_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    opts="add connect list delete update"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
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
