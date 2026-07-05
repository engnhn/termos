#compdef termos

_termos() {
    local -a commands
    commands=(
        'add:Add a new server connection'
        'connect:Establish an SSH connection to a saved server'
        'list:List all registered servers'
        'delete:Delete a registered server connection'
        'update:Update Termos to the latest version'
        'quick-command:Manage quick commands for a server'
        'qc:Manage quick commands for a server (alias)'
    )

    if (( CURRENT == 2 )); then
        _describe -t commands 'termos commands' commands
    elif (( CURRENT == 3 )); then
        case $words[2] in
            connect|delete)
                local -a nicknames
                nicknames=(${(f)"$(termos _list-nicknames 2>/dev/null)"})
                _describe -t nicknames 'saved servers' nicknames
                ;;
            quick-command|qc)
                local -a qc_subs
                qc_subs=(
                    'list:List quick commands'
                    'add:Add a quick command'
                    'edit:Edit a quick command'
                    'delete:Delete a quick command'
                )
                _describe -t qc_subs 'quick-command subcommands' qc_subs
                ;;
        esac
    elif (( CURRENT == 4 )); then
        case $words[2] in
            quick-command|qc)
                case $words[3] in
                    list|add|edit|delete)
                        local -a nicknames
                        nicknames=(${(f)"$(termos _list-nicknames 2>/dev/null)"})
                        _describe -t nicknames 'saved servers' nicknames
                        ;;
                esac
                ;;
        esac
    fi
}

_termos "$@"
