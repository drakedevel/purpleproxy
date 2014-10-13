#!/bin/bash

PIDGIN_DIR=${1:-~/pidgin/src/pidgin-2.10.9}

./protos $PIDGIN_DIR/libpurple/purple.h -- \
        $(pkg-config --cflags glib-2.0) \
        -I$PIDGIN_DIR/libpurple \
        -I/usr/lib/clang/$(llvm-config --version)/include \
    | sed -n -e '/^\s*$/d' -e '/purple_value_new/,$p' \
    | sed -e 's/)$/);\n/' \
    | head -n -1
