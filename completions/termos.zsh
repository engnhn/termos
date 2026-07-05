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
        'usage:View detailed interactive user manual and usage guide'
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
            connect)
                local -a conn_opts
                conn_opts=(
                    '-q:Optional quick command name'
                    '--qc:Optional quick command name'
                )
                _describe -t conn_opts 'connection options' conn_opts
                ;;
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
    elif (( CURRENT >= 5 )); then
        case $words[2] in
            connect)
                local -a conn_opts
                conn_opts=(
                    '-q:Optional quick command name'
                    '--qc:Optional quick command name'
                )
                _describe -t conn_opts 'connection options' conn_opts
                ;;
            quick-command|qc)
                case $words[3] in
                    add)
                        local -a add_opts
                        add_opts=(
                            '--name:Name of the quick command'
                            '--cmd:The command string to run'
                        )
                        _describe -t add_opts 'add options' add_opts
                        ;;
                    edit)
                        local -a edit_opts
                        edit_opts=(
                            '--name:Current name of the quick command'
                            '--new-name:New name'
                            '--new-cmd:New command string'
                        )
                        _describe -t edit_opts 'edit options' edit_opts
                        ;;
                    delete)
                        local -a delete_opts
                        delete_opts=(
                            '--name:Name of the quick command to delete'
                        )
                        _describe -t delete_opts 'delete options' delete_opts
                        ;;
                esac
                ;;
        esac
    fi
}

_termos "$@"
