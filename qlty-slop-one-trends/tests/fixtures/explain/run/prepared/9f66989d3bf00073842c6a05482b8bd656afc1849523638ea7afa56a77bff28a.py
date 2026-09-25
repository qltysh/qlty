def render(rows, verbose, color, width):
    out = []
    for row in rows:
        if row.ok and not row.hidden and (verbose or row.level > 1) and color:
            out.append(f"{row.name}: {row.value}")
        elif row.ok:
            out.append(row.name)
    return "\n".join(out)


def report_0(record, options):
    if record.kind == 0:
        return options.render(record, 0)
    return None


def report_1(record, options):
    if record.kind == 1:
        return options.render(record, 1)
    return None


def report_2(record, options):
    if record.kind == 2:
        return options.render(record, 2)
    return None


def report_3(record, options):
    if record.kind == 3:
        return options.render(record, 3)
    return None


def report_4(record, options):
    if record.kind == 4:
        return options.render(record, 4)
    return None


def report_5(record, options):
    if record.kind == 5:
        return options.render(record, 5)
    return None


def report_6(record, options):
    if record.kind == 6:
        return options.render(record, 6)
    return None


def report_7(record, options):
    if record.kind == 7:
        return options.render(record, 7)
    return None


def report_8(record, options):
    if record.kind == 8:
        return options.render(record, 8)
    return None
