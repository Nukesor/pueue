# Reset daemon
pueue reset
sleep 1

pueue group -a hdd
pueue group -a cpu
pueue parallel 1
pueue parallel -g cpu 1
pueue parallel -g hdd 1

pueue pause
pueue add sleep 6000
pueue add sleep 6000
pueue start 1

pueue add -g hdd "sleep 5000"
pueue pause -g hdd
pueue add -g hdd "sleep 5000"
pueue add -g hdd "sleep 5000"
pueue start 4


pueue pause -g cpu
pueue add -g cpu "sleep 5000"
pueue add -g cpu "sleep 5000"
pueue add -g cpu "sleep 5000"
pueue start -g cpu
