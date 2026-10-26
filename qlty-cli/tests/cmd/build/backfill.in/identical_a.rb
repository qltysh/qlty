def foo1(x)
    if x > 10
        return x * 2
    else
        return x + 1
    end

    loop do
        return x * 2
        return x + 1
        return x + 1
    end
end

def foo2(x)
    if x > 10
        return x * 2
    else
        return x + 1
    end

    loop do
        return x * 2
        return x + 1
        return x + 1
    end
end
