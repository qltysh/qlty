def connect(host, port, user, password, timeout, retries, tls):
    return Client(host, port, user, password, timeout, retries, tls)


def reconnect(host, port, user, password, timeout, retries, tls):
    return Client(host, port, user, password, timeout, retries, tls)


def connection_0(record, options):
    if record.kind == 0:
        return options.render(record, 0)
    return None


def connection_1(record, options):
    if record.kind == 1:
        return options.render(record, 1)
    return None


def connection_2(record, options):
    if record.kind == 2:
        return options.render(record, 2)
    return None


def connection_3(record, options):
    if record.kind == 3:
        return options.render(record, 3)
    return None


def connection_4(record, options):
    if record.kind == 4:
        return options.render(record, 4)
    return None


def connection_5(record, options):
    if record.kind == 5:
        return options.render(record, 5)
    return None
