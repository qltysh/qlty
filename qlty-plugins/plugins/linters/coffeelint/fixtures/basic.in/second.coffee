# You would expect this to result in 8, but
# it might result in 0 (parsed as octal).
parseInt '08'

# To be safe, specify the radix argument:
parseInt '08', 10


# CoffeeLint will catch this:
throw "i made a boo boo"

# ... but not this:
throw getSomeString()
