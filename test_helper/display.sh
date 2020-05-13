pueue reset
sleep 1
pueue add ls
pueue add failing
pueue add sleep 6000
pueue add ls
pueue add sleep 6000
sleep 0.5
pueue start 4
sleep 0.5
pueue kill 4

sleep 0.5
pueue status
