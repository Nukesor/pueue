pueue add ls
pueue status
pueue log
pueue add sleep 120
pueue add failing_command
pueue status

# Clear screen
pueue add ./test_helper/follow.sh
pueue status
pueue parallel 2
pueue status
pueue follow 3
pueue follow 3 -e
