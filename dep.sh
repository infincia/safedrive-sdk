#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

mkdir -p dep/$TARGET/lib
mkdir -p dep/$TARGET/include
mkdir -p dep/$TARGET/bin

# these are at the top for visibility, changing a version will always cause a rebuild, otherwise
# they will only be rebuilt if the built product is not found
export OPENSSL_VER=1.1.0e
export OPENSSL_VER_FILE=$PWD/dep/${TARGET}/lib/.openssl_ver

export SODIUM_VER=1.0.11
export SODIUM_VER_FILE=$PWD/dep/${TARGET}/lib/.sodium_ver

export LIBDBUS_VER=1.10.14
export LIBDBUS_VER_FILE=$PWD/dep/${TARGET}/lib/.dbus_ver

export BUILD_DBUS=false

export BUILD_OPENSSL=false

export BUILD_LIBSODIUM=false

case ${TARGET} in
    x86_64-apple-darwin)
        export OSX_VERSION_MIN=${OSX_VERSION_MIN-"10.9"}
        export OSX_CPU_ARCH=${OSX_CPU_ARCH-"core2"}
        export CFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -O2 -g -flto"
        export LDFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -flto"
        export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"
        export SODIUM_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="no-deprecated shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        export LIBDBUS_ARGS="--enable-shared=yes"
        BUILD_LIBSODIUM=true
        ;;
    x86_64-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export SODIUM_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="no-deprecated shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        export LIBDBUS_ARGS="--enable-shared=yes"
        BUILD_DBUS=true
        BUILD_OPENSSL=true
        BUILD_LIBSODIUM=true
        ;;
    i686-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto -m32"
        export LDFLAGS="-flto"
        export SODIUM_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="no-deprecated shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        export LIBDBUS_ARGS="--enable-shared=yes"
        export PKG_CONFIG_ALLOW_CROSS=1
        BUILD_DBUS=true
        BUILD_OPENSSL=true
        BUILD_LIBSODIUM=true
        ;;
    x86_64-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        export SODIUM_ARGS="--enable-shared=no"
        export OPENSSL_ARGS="no-deprecated no-shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        export LIBDBUS_ARGS="--enable-shared=no"
        BUILD_DBUS=true
        BUILD_OPENSSL=true
        BUILD_LIBSODIUM=true
        ;;
    i686-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto -m32"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        export SODIUM_ARGS="--enable-shared=no"
        export OPENSSL_ARGS="no-deprecated no-shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        export LIBDBUS_ARGS="--enable-shared=no"
        export PKG_CONFIG_ALLOW_CROSS=1
        BUILD_DBUS=true
        BUILD_OPENSSL=true
        BUILD_LIBSODIUM=true
        ;;
    *)
        ;;
esac

if [ ! -f dep/${TARGET}/lib/libdbus-1.a ] || [ ! -f ${LIBDBUS_VER_FILE} ] || [ ! $(<${LIBDBUS_VER_FILE}) = ${LIBDBUS_VER} ]; then
    if [ ${BUILD_DBUS} = true ]; then
        LIBDBUS_PREFIX=$PWD/dep/${TARGET}

        echo "Building libdbus ${LIBDBUS_VER} for ${TARGET} in ${LIBDBUS_PREFIX}"

        echo ${LIBDBUS_VER} > ${LIBDBUS_VER_FILE}

        wget https://dbus.freedesktop.org/releases/dbus/dbus-${LIBDBUS_VER}.tar.gz > /dev/null
        tar xvzf dbus-${LIBDBUS_VER}.tar.gz > /dev/null
        pushd dbus-${LIBDBUS_VER}
        ./configure --prefix=${LIBDBUS_PREFIX} ${LIBDBUS_ARGS} > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf dbus*
    else
        echo "Not building libdbus"
    fi
else
    echo "Not building libdbus"
fi

if [ ! -f dep/${TARGET}/lib/libssl.a ] || [ ! -f ${OPENSSL_VER_FILE} ] || [ ! $(<${OPENSSL_VER_FILE}) = ${OPENSSL_VER} ]; then
    if [ ${BUILD_OPENSSL} = true ]; then
        OPENSSL_PREFIX=$PWD/dep/${TARGET}

        echo "Building OpenSSL ${OPENSSL_VER} for ${TARGET} in ${OPENSSL_PREFIX}"

        echo ${OPENSSL_VER} > ${OPENSSL_VER_FILE}

        wget https://www.openssl.org/source/openssl-${OPENSSL_VER}.tar.gz > /dev/null
        tar xvzf openssl-${OPENSSL_VER}.tar.gz > /dev/null
        pushd openssl-${OPENSSL_VER}
        ./config --prefix=${OPENSSL_PREFIX} --openssldir=/usr/lib/ssl ${OPENSSL_ARGS} > /dev/null
        make clean > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf openssl*
    else
        echo "Not building openssl"
    fi
else
    echo "Not building openssl"
fi

if [ ! -f dep/${TARGET}/lib/libsodium.a ] || [ ! -f ${SODIUM_VER_FILE} ] || [ ! $(<${SODIUM_VER_FILE}) = ${SODIUM_VER} ]; then
    if [ ${BUILD_LIBSODIUM} = true ]; then
        SODIUM_PREFIX=$PWD/dep/${TARGET}

        echo "Building libsodium ${SODIUM_VER} for ${TARGET} in ${SODIUM_PREFIX}"

        echo ${SODIUM_VER} > ${SODIUM_VER_FILE}

        wget https://github.com/jedisct1/libsodium/releases/download/${SODIUM_VER}/libsodium-${SODIUM_VER}.tar.gz > /dev/null
        tar xvfz libsodium-${SODIUM_VER}.tar.gz > /dev/null
        pushd libsodium-${SODIUM_VER}
        ./configure --prefix=${SODIUM_PREFIX} ${SODIUM_ARGS} > /dev/null
        make clean > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf libsodium*
    else
        echo "Not building libsodium"
    fi
else
    echo "Not building libsodium"
fi