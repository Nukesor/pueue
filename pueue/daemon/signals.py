import signal

signals = {
    # SigInt
    '2': signal.SIGINT,
    'sigint': signal.SIGINT,
    'int': signal.SIGINT,

    # SigKill
    '9': signal.SIGKILL,
    'sigkill': signal.SIGKILL,
    'kill': signal.SIGKILL,

    # SigTerm
    '15': signal.SIGTERM,
    'sigterm': signal.SIGTERM,
    'term': signal.SIGTERM,
}
