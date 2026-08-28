#!/bin/sh
echo '#!/bin/sh' > greet.sh
echo 'grep -q "hello" "$1" && exit 0 || exit 1' >> greet.sh
chmod +x greet.sh
