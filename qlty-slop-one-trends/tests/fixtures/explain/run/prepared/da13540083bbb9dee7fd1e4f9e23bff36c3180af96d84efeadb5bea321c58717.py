def greet(name):
    return "hello " + name


def greeting_0(record, options):
    if record.kind == 0:
        return options.render(record, 0)
    return None


def greeting_1(record, options):
    if record.kind == 1:
        return options.render(record, 1)
    return None


def greeting_2(record, options):
    if record.kind == 2:
        return options.render(record, 2)
    return None
