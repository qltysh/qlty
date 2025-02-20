#/usr/bin/env bash
rm -fr ~/.cache/qlty/tools/ruby ~/.cache/qlty/tools/rubocop

export RUNTIME="ruby"
export RUNTIME_VERSION="3.2.2"

runtime_directory="/Users/bhelmkamp/.cache/qlty/tools/$RUNTIME/$RUNTIME_VERSION"
mkdir -p $runtime_directory
cd $runtime_directory || exit

download_filename="v20230330.tar.gz"
download_url="https://github.com/rbenv/ruby-build/archive/refs/tags/$download_filename"
wget "$download_url"
tar --strip-components=1 -xpvzf $download_filename
rm $download_filename
