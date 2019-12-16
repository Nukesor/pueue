
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'pueue' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'pueue'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-')) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'pueue' {
            [CompletionResult]::new('-a', 'a', [CompletionResultType]::ParameterName, 'The url for the daemon. Overwrites the address in the config file')
            [CompletionResult]::new('--address', 'address', [CompletionResultType]::ParameterName, 'The url for the daemon. Overwrites the address in the config file')
            [CompletionResult]::new('-p', 'p', [CompletionResultType]::ParameterName, 'The port for the daemon. Overwrites the port in the config file')
            [CompletionResult]::new('--port', 'port', [CompletionResultType]::ParameterName, 'The port for the daemon. Overwrites the port in the config file')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Verbose mode (-v, -vv, -vvv)')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Verbose mode (-v, -vv, -vvv)')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Enqueue a task for execution')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a tasks from the list. Running or paused tasks need to be killed first.')
            [CompletionResult]::new('switch', 'switch', [CompletionResultType]::ParameterValue, 'Switches the queue position of two commands. Only works on queued and stashed commands')
            [CompletionResult]::new('stash', 'stash', [CompletionResultType]::ParameterValue, 'Stashed tasks won''t be automatically started. Either `enqueue` them, to be normally handled or explicitely `start` them.')
            [CompletionResult]::new('enqueue', 'enqueue', [CompletionResultType]::ParameterValue, 'Enqueue stashed tasks. They''ll be handled normally afterwards.')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Wake the daemon from its paused state. Also continues all paused tasks.')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Enqueue tasks again.')
            [CompletionResult]::new('pause', 'pause', [CompletionResultType]::ParameterValue, 'Pause the daemon and all running tasks. A paused daemon won''t start any new tasks. Daemon and tasks can be continued with `start`')
            [CompletionResult]::new('kill', 'kill', [CompletionResultType]::ParameterValue, 'Kill running tasks.')
            [CompletionResult]::new('send', 'send', [CompletionResultType]::ParameterValue, 'Send something to a task. Useful for sending confirmations (''y\n'')')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Edit the command of a stashed or queued task.')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Display the current status of all tasks')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Display the log output of finished tasks')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show the output of a currently running task This command allows following (like `tail -f`)')
            [CompletionResult]::new('reset', 'reset', [CompletionResultType]::ParameterValue, 'Kill all running tasks, remove all tasks and reset max_id.')
            [CompletionResult]::new('clean', 'clean', [CompletionResultType]::ParameterValue, 'Remove all finished tasks from the list (also clears logs).')
            [CompletionResult]::new('parallel', 'parallel', [CompletionResultType]::ParameterValue, 'Set the amount of allowed parallel tasks')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Prints this message or the help of the given subcommand(s)')
            break
        }
        'pueue;add' {
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Start the task immediately')
            [CompletionResult]::new('--immediate', 'immediate', [CompletionResultType]::ParameterName, 'Start the task immediately')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;remove' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;switch' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;stash' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;enqueue' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;start' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Enforce starting these tasks. This doesn''t affect the daemon or any other tasks and works on a paused deamon.')
            [CompletionResult]::new('--task-ids', 'task-ids', [CompletionResultType]::ParameterName, 'Enforce starting these tasks. This doesn''t affect the daemon or any other tasks and works on a paused deamon.')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;restart' {
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Start the task(s) immediately')
            [CompletionResult]::new('--immediate', 'immediate', [CompletionResultType]::ParameterName, 'Start the task(s) immediately')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;pause' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Enforce starting these tasks. Doesn''t affect the daemon or any other tasks.')
            [CompletionResult]::new('--task-ids', 'task-ids', [CompletionResultType]::ParameterName, 'Enforce starting these tasks. Doesn''t affect the daemon or any other tasks.')
            [CompletionResult]::new('-w', 'w', [CompletionResultType]::ParameterName, 'Pause the daemon, but let any running tasks finish by themselves.')
            [CompletionResult]::new('--wait', 'wait', [CompletionResultType]::ParameterName, 'Pause the daemon, but let any running tasks finish by themselves.')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;kill' {
            [CompletionResult]::new('-a', 'a', [CompletionResultType]::ParameterName, 'Kill all running tasks, this also pauses the daemon.')
            [CompletionResult]::new('--all', 'all', [CompletionResultType]::ParameterName, 'Kill all running tasks, this also pauses the daemon.')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;send' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;edit' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;status' {
            [CompletionResult]::new('-j', 'j', [CompletionResultType]::ParameterName, 'Print the current state as json to stdout This doesn''t include stdout/stderr of tasks. Use `log -j` if you want everything')
            [CompletionResult]::new('--json', 'json', [CompletionResultType]::ParameterName, 'Print the current state as json to stdout This doesn''t include stdout/stderr of tasks. Use `log -j` if you want everything')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;log' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Specify for which specific tasks you want to see the output')
            [CompletionResult]::new('--task-ids', 'task-ids', [CompletionResultType]::ParameterName, 'Specify for which specific tasks you want to see the output')
            [CompletionResult]::new('-j', 'j', [CompletionResultType]::ParameterName, 'Print the current state as json Includes EVERYTHING')
            [CompletionResult]::new('--json', 'json', [CompletionResultType]::ParameterName, 'Print the current state as json Includes EVERYTHING')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;show' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Continuously print stdout (like `tail -f`)')
            [CompletionResult]::new('--follow', 'follow', [CompletionResultType]::ParameterName, 'Continuously print stdout (like `tail -f`)')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'Like -f, but shows stderr instead of stdeout.')
            [CompletionResult]::new('--err', 'err', [CompletionResultType]::ParameterName, 'Like -f, but shows stderr instead of stdeout.')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;reset' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;clean' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;parallel' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
        'pueue;help' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Prints help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Prints version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Prints version information')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
