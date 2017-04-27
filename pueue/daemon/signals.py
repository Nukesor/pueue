import signal

signals = {
    # SigHup
    '1': signal.SIGHUP,
    'sighup': signal.SIGHUP,
    'hup': signal.SIGHUP,

    # SigInt
    '2': signal.SIGINT,
    'sigint': signal.SIGINT,
    'int': signal.SIGINT,

    # SigQuit
    '3': signal.SIGQUIT,
    'sigquit': signal.SIGQUIT,
    'quit': signal.SIGQUIT,

    # SigKill
    '9': signal.SIGKILL,
    'sigkill': signal.SIGKILL,
    'kill': signal.SIGKILL,

    # SigTerm
    '15': signal.SIGTERM,
    'sigterm': signal.SIGTERM,
    'term': signal.SIGTERM,

    # SigCont
    '18': signal.SIGCONT,
    'sigcont': signal.SIGCONT,
    'cont': signal.SIGCONT,

    # SigStop
    '19': signal.SIGSTOP,
    'sigstop': signal.SIGSTOP,
    'stop': signal.SIGSTOP,
}
