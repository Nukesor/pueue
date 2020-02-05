counter = 1

while true; do
    echo "nice ${counter}"
    >&2 echo "error ${counter}"
    ((counter++))

    sleep 1
done
