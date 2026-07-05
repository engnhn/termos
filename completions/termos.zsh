#compdef termos

_termos() {
    local -a commands
    commands=(
        'add:Add a new server connection'
        'connect:Establish an SSH connection to a saved server'
        'list:List all registered servers'
        'delete:Delete a registered server connection'
        'update:Update Termos to the latest version'
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
        esac
    fi
}

_termos "$@"
