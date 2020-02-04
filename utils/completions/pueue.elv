
edit:completion:arg-completer[pueue] = [@words]{
    fn spaces [n]{
        repeat $n ' ' | joins ''
    }
    fn cand [text desc]{
        edit:complex-candidate $text &display-suffix=' '(spaces (- 14 (wcswidth $text)))$desc
    }
    command = 'pueue'
    for word $words[1:-1] {
        if (has-prefix $word '-') {
            break
        }
        command = $command';'$word
    }
    completions = [
        &'pueue'= {
            cand -p 'The port for the daemon. Overwrites the port in the config file'
            cand --port 'The port for the daemon. Overwrites the port in the config file'
            cand -v 'Verbose mode (-v, -vv, -vvv)'
            cand --verbose 'Verbose mode (-v, -vv, -vvv)'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
            cand add 'Enqueue a task for execution'
            cand remove 'Remove tasks from the list. Running or paused tasks need to be killed first'
            cand switch 'Switches the queue position of two commands. Only works on queued and stashed commandse'
            cand stash 'Stashed tasks won''t be automatically started. Either `enqueue` them, to be normally handled or explicitly `start` them'
            cand enqueue 'Enqueue stashed tasks. They''ll be handled normally afterwards'
            cand start 'Wake the daemon from its paused state and continue all paused tasks. Can be used to resume or start specific tasks'
            cand restart 'Enqueue tasks again'
            cand pause 'Pause the daemon and all running tasks. A paused daemon won''t start any new tasks. Daemon and tasks can be continued with `start` Can also be used to pause specific tasks'
            cand kill 'Kill either all or only specific running tasks'
            cand send 'Send something to a task. Useful for sending confirmations (''y\n'')'
            cand edit 'Edit the command of a stashed or queued task'
            cand status 'Display the current status of all tasks'
            cand log 'Display the log output of finished tasks'
            cand show 'Show the output of a currently running task This command allows following (like `tail -f`)'
            cand clean 'Remove all finished tasks from the list (also clears logs)'
            cand reset 'Kill all running tasks, remove all tasks and reset max_id'
            cand shutdown 'Remotely shut down the daemon. Should only be used if the daemon isn''t started by a service manager'
            cand parallel 'Set the amount of allowed parallel tasks'
            cand help 'Prints this message or the help of the given subcommand(s)'
        }
        &'pueue;add'= {
            cand -i 'Start the task immediately'
            cand --immediate 'Start the task immediately'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;remove'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;switch'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;stash'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;enqueue'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;start'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;restart'= {
            cand -i 'Start the task(s) immediately'
            cand --immediate 'Start the task(s) immediately'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;pause'= {
            cand -w 'Pause the daemon, but let any running tasks finish by themselves'
            cand --wait 'Pause the daemon, but let any running tasks finish by themselves'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;kill'= {
            cand -a 'Kill all running tasks, this also pauses the daemon'
            cand --all 'Kill all running tasks, this also pauses the daemon'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;send'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;edit'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;status'= {
            cand -j 'Print the current state as json to stdout This doesn''t include stdout/stderr of tasks. Use `log -j` if you want everything'
            cand --json 'Print the current state as json to stdout This doesn''t include stdout/stderr of tasks. Use `log -j` if you want everything'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;log'= {
            cand -j 'Print the current state as json Includes EVERYTHING'
            cand --json 'Print the current state as json Includes EVERYTHING'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;show'= {
            cand -f 'Continuously print stdout (like `tail -f`)'
            cand --follow 'Continuously print stdout (like `tail -f`)'
            cand -e 'Like -f, but shows stderr instead of stdeout'
            cand --err 'Like -f, but shows stderr instead of stdeout'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;clean'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;reset'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;shutdown'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;parallel'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
        &'pueue;help'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
        }
    ]
    $completions[$command]
}
