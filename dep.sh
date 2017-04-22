#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

export BUILD_PREFIX=${PWD}/dep/${TARGET}
export DIST_PREFIX=${PWD}/dist-${TARGET}

export PATH=${BUILD_PREFIX}/bin:${PATH}

mkdir -p ${BUILD_PREFIX}/lib
mkdir -p ${BUILD_PREFIX}/include
mkdir -p ${BUILD_PREFIX}/bin

mkdir -p ${DIST_PREFIX}/lib
mkdir -p ${DIST_PREFIX}/include
mkdir -p ${DIST_PREFIX}/bin

mkdir -p src
mkdir -p build

# grab sources

pushd src > /dev/null

if [ ! -f expat-${EXPAT_VER}.tar.bz2 ]; then
    echo "Downloading expat-${EXPAT_VER}.tar.bz2"
    echo "From https://downloads.sourceforge.net/project/expat/expat/${EXPAT_VER}/expat-${EXPAT_VER}.tar.bz2"
    curl -L https://downloads.sourceforge.net/project/expat/expat/${EXPAT_VER}/expat-${EXPAT_VER}.tar.bz2 -o expat-${EXPAT_VER}.tar.bz2 > /dev/null

fi

if [ ! -f dbus-${LIBDBUS_VER}.tar.gz ]; then
    echo "Downloading dbus-${LIBDBUS_VER}.tar.gz"
    echo "From https://dbus.freedesktop.org/releases/dbus/dbus-${LIBDBUS_VER}.tar.gz"
    curl -L https://dbus.freedesktop.org/releases/dbus/dbus-${LIBDBUS_VER}.tar.gz -o dbus-${LIBDBUS_VER}.tar.gz> /dev/null
fi

if [ ! -f libffi-${FFI_VER}.tar.gz ]; then
    echo "Downloading libffi-${FFI_VER}.tar.gz"
    echo "From ftp://sourceware.org/pub/libffi/libffi-${FFI_VER}.tar.gz"
    curl -L ftp://sourceware.org/pub/libffi/libffi-${FFI_VER}.tar.gz -o libffi-${FFI_VER}.tar.gz > /dev/null
fi

if [ ! -f libsodium-${SODIUM_VER}.tar.gz ]; then
    echo "Downloading libsodium-${SODIUM_VER}.tar.gz"
    echo "From https://github.com/jedisct1/libsodium/releases/download/${SODIUM_VER}/libsodium-${SODIUM_VER}.tar.gz"
    curl -L https://github.com/jedisct1/libsodium/releases/download/${SODIUM_VER}/libsodium-${SODIUM_VER}.tar.gz -o libsodium-${SODIUM_VER}.tar.gz > /dev/null
fi

if [ ! -f libressl-${LIBRESSL_VER}.tar.gz ]; then
    echo "Downloading libressl-${LIBRESSL_VER}.tar.gz"
    curl -L https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VER}.tar.gz -o libressl-${LIBRESSL_VER}.tar.gz > /dev/null
    curl -L https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VER}.tar.gz.asc -o libressl-${LIBRESSL_VER}.tar.gz.asc > /dev/null
fi

popd > /dev/null


# these are at the top for visibility, changing a version will always cause a rebuild, otherwise
# they will only be rebuilt if the built product is not found
export SODIUM_VER=1.0.12
export SODIUM_VER_FILE=${BUILD_PREFIX}/.sodium_ver
export SODIUM_ARGS="--enable-shared=no"

export LIBDBUS_VER=1.10.14
export LIBDBUS_VER_FILE=${BUILD_PREFIX}/.dbus_ver
export LIBDBUS_ARGS="--enable-shared=no --disable-tests --with-x=no --disable-systemd --disable-launchd --disable-libaudit --disable-selinux --disable-apparmor"

export EXPAT_VER=2.2.0
export EXPAT_VER_FILE=${BUILD_PREFIX}/.expat_ver
export EXPAT_ARGS="--enable-shared=no"


export LIBRESSL_VER=2.5.2
export LIBRESSL_VER_FILE=${BUILD_PREFIX}/.libressl_ver
export LIBRESSL_ARGS="--disable-dependency-tracking"

export FFI_VER=3.2.1
export FFI_VER_FILE=${BUILD_PREFIX}/.ffi_ver
export FFI_ARGS=""

export ac_cv_func_timingsafe_bcmp="no"
export ac_cv_func_basename_r="no"
export ac_cv_func_clock_getres="no"
export ac_cv_func_clock_gettime="no"
export ac_cv_func_clock_settime="no"
export ac_cv_func_dirname_r="no"
export ac_cv_func_getentropy="no"
export ac_cv_func_mkostemp="no"
export ac_cv_func_mkostemps="no"


export BUILD_EXPAT=false
export BUILD_DBUS=false
export BUILD_LIBSODIUM=true
export BUILD_LIBRESSL=false
export BUILD_FFI=false

export RUSTFLAGS=""
export CFLAGS="-O2 -g -I${BUILD_PREFIX}/include"
export CPPFLAGS="-O2 -g -I${BUILD_PREFIX}/include"
export LDFLAGS="-L${BUILD_PREFIX}/lib"

export MACOSX_DEPLOYMENT_TARGET=10.9
export OSX_VERSION_MIN="10.9"
export OSX_CPU_ARCH="core2"
export MAC_ARGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH}"

case ${TARGET} in
    x86_64-apple-darwin)
        export CFLAGS="${CFLAGS} ${MAC_ARGS}"
        export CPPFLAGS="${CPPFLAGS} ${MAC_ARGS}"
        export LDFLAGS="${LDFLAGS} ${MAC_ARGS}"
        export RUSTFLAGS="${RUSTFLAGS} -C link-args=-mmacosx-version-min=${OSX_VERSION_MIN}"
        export BUILD_LIBRESSL=true
        export BUILD_FFI=true
        ;;
    x86_64-unknown-linux-gnu)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        export BUILD_EXPAT=true
        export BUILD_DBUS=true
        ;;
    i686-unknown-linux-gnu)
        export CFLAGS="${CFLAGS} -m32"
        export CPPFLAGS="${CPPFLAGS} -m32"
        export LDFLAGS="${LDFLAGS}"
        export PKG_CONFIG_ALLOW_CROSS=1
        export BUILD_EXPAT=true
        export BUILD_DBUS=true
        ;;
    x86_64-unknown-linux-musl)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        export CC=musl-gcc
        export BUILD_EXPAT=true
        export BUILD_DBUS=true
        ;;
    i686-unknown-linux-musl)
        export CFLAGS="${CFLAGS} -m32"
        export CPPFLAGS="${CPPFLAGS} -m32"
        export LDFLAGS="${LDFLAGS}"
        export CC=musl-gcc
        export PKG_CONFIG_ALLOW_CROSS=1
        export BUILD_EXPAT=true
        export BUILD_DBUS=true
        ;;
    *)
        ;;
esac









pushd build > /dev/null

if [ ! -f ${BUILD_PREFIX}/lib/libexpat.a ] || [ ! -f ${EXPAT_VER_FILE} ] || [ ! $(<${EXPAT_VER_FILE}) = ${EXPAT_VER} ]; then
    if [ ${BUILD_EXPAT} = true ]; then

        echo "Building libexpat ${EXPAT_VER} for ${TARGET} in ${BUILD_PREFIX}"
        tar xf ../src/expat-${EXPAT_VER}.tar.bz2 > /dev/null
        pushd expat-${EXPAT_VER}
        ./configure --prefix=${BUILD_PREFIX} ${EXPAT_ARGS} > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf expat*
        echo ${EXPAT_VER} > ${EXPAT_VER_FILE}
    else
        echo "Not set to build libexpat"
    fi
else
    echo "Not building libexpat"
fi

if [ ! -f ${BUILD_PREFIX}/lib/libdbus-1.a ] || [ ! -f ${LIBDBUS_VER_FILE} ] || [ ! $(<${LIBDBUS_VER_FILE}) = ${LIBDBUS_VER} ]; then
    if [ ${BUILD_DBUS} = true ]; then
        echo "Building libdbus ${LIBDBUS_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/dbus-${LIBDBUS_VER}.tar.gz > /dev/null
        pushd dbus-${LIBDBUS_VER}
        ./configure --prefix=${BUILD_PREFIX} ${LIBDBUS_ARGS} > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf dbus*
        echo ${LIBDBUS_VER} > ${LIBDBUS_VER_FILE}
    else
        echo "Not set to build libdbus"
    fi
else
    echo "Not building libdbus"
fi

if [ ! -f d${BUILD_PREFIX}/lib/libsodium.a ] || [ ! -f ${SODIUM_VER_FILE} ] || [ ! $(<${SODIUM_VER_FILE}) = ${SODIUM_VER} ]; then
    if [ ${BUILD_LIBSODIUM} = true ]; then

        echo "Building libsodium ${SODIUM_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/libsodium-${SODIUM_VER}.tar.gz > /dev/null
        pushd libsodium-${SODIUM_VER}
        ./configure --prefix=${BUILD_PREFIX} ${SODIUM_ARGS} > /dev/null
        make clean > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf libsodium*
        echo ${SODIUM_VER} > ${SODIUM_VER_FILE}

    else
        echo "Not set to build libsodium"
    fi
else
    echo "Not building libsodium"
fi

if [ ! -f ${BUILD_PREFIX}/lib/libssl.a ] || [ ! -f ${LIBRESSL_VER_FILE} ] || [ ! $(<${LIBRESSL_VER_FILE}) = ${LIBRESSL_VER} ]; then
    if [ ${BUILD_LIBRESSL} = true ]; then

        echo "Building LibreSSL ${LIBRESSL_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf libressl*
        tar xf ../src/libressl-${LIBRESSL_VER}.tar.gz > /dev/null
        pushd libressl-${LIBRESSL_VER} > /dev/null
            ./configure --prefix=${BUILD_PREFIX} ${LIBRESSL_ARGS} > /dev/null
            make check > /dev/null
            make install > /dev/null
        popd > /dev/null
        rm -rf libressl*
        echo ${LIBRESSL_VER} > ${LIBRESSL_VER_FILE}
    else
        echo "Not set to build LibreSSL"
    fi
else
    echo "Not building LibreSSL"
fi

if [ ! -f  ${BUILD_PREFIX}/lib/libffi.a ] || [ ! -f ${FFI_VER_FILE} ] || [ ! $(<${FFI_VER_FILE}) = ${FFI_VER} ]; then
    if [ ${BUILD_FFI} = true ]; then

        echo "Building libffi ${FFI_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/libffi-${FFI_VER}.tar.gz > /dev/null
        pushd libffi-${FFI_VER}
            ./configure --prefix=${BUILD_PREFIX} ${FFI_ARGS} > /dev/null
            make > /dev/null
            make install > /dev/null
        popd
        rm -rf libffi*
        echo ${FFI_VER} > ${FFI_VER_FILE}
    else
        echo "Not set to build libffi"
    fi
else
    echo "Not building libffi"
fi