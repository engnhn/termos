_termos_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    opts="add connect list delete update quick-command qc usage"

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
        elif [[ ${COMP_CWORD} -ge 4 ]]; then
            local sub="${COMP_WORDS[2]}"
            local qc_opts=""
            case "${sub}" in
                add)
                    qc_opts="--name --cmd"
                    ;;
                edit)
                    qc_opts="--name --new-name --new-cmd"
                    ;;
                delete)
                    qc_opts="--name"
                    ;;
            esac
            local filtered_opts=""
            for opt in ${qc_opts}; do
                if [[ ! " ${COMP_WORDS[@]} " =~ " ${opt} " ]]; then
                    filtered_opts="${filtered_opts} ${opt}"
                fi
            done
            COMPREPLY=( $(compgen -W "${filtered_opts}" -- ${cur}) )
            return 0
        fi
    fi

    if [[ "${cmd}" == "connect" ]]; then
        if [[ ${COMP_CWORD} -eq 2 ]]; then
            local nicknames=$(termos _list-nicknames 2>/dev/null)
            COMPREPLY=( $(compgen -W "${nicknames}" -- ${cur}) )
            return 0
        elif [[ ${COMP_CWORD} -ge 3 ]]; then
            local conn_opts="--qc -q"
            local filtered_opts=""
            for opt in ${conn_opts}; do
                if [[ ! " ${COMP_WORDS[@]} " =~ " ${opt} " ]]; then
                    filtered_opts="${filtered_opts} ${opt}"
                fi
            done
            COMPREPLY=( $(compgen -W "${filtered_opts}" -- ${cur}) )
            return 0
        fi
    fi

    if [[ "${cmd}" == "delete" ]]; then
        if [[ ${COMP_CWORD} -eq 2 ]]; then
            local nicknames=$(termos _list-nicknames 2>/dev/null)
            COMPREPLY=( $(compgen -W "${nicknames}" -- ${cur}) )
            return 0
        fi
    fi
}

complete -F _termos_completions termos
