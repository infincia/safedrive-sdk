#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

export BUILD_PREFIX=${PWD}/dep/${TARGET}
export DIST_PREFIX=${PWD}/dist-${TARGET}


mkdir -p ${BUILD_PREFIX}/lib
mkdir -p ${BUILD_PREFIX}/include
mkdir -p ${BUILD_PREFIX}/bin

mkdir -p ${DIST_PREFIX}/lib
mkdir -p ${DIST_PREFIX}/include
mkdir -p ${DIST_PREFIX}/bin

mkdir -p src
mkdir -p build

# these are at the top for visibility, changing a version will always cause a rebuild, otherwise
# they will only be rebuilt if the built product is not found
export SODIUM_VER=1.0.12
export SODIUM_VER_FILE=${BUILD_PREFIX}/.sodium_ver
export SODIUM_ARGS="--enable-shared=no --enable-static"

export LIBDBUS_VER=1.10.18
export LIBDBUS_VER_FILE=${BUILD_PREFIX}/.dbus_ver
export LIBDBUS_ARGS="--enable-shared=no --enable-static --disable-tests --with-x=no --disable-systemd --disable-launchd --disable-libaudit --disable-selinux --disable-apparmor"

export EXPAT_VER=2.2.0
export EXPAT_VER_FILE=${BUILD_PREFIX}/.expat_ver
export EXPAT_ARGS="--enable-shared=no --enable-static"

export ICONV_VER=1.15
export ICONV_VER_FILE=${BUILD_PREFIX}/.iconv_ver
export ICONV_ARGS="--enable-shared=no --enable-static"
export LIBICONV_CFLAGS=-I${BUILD_PREFIX}/include
export LIBICONV_LIBS=-L${BUILD_PREFIX}/lib

export GETTEXT_VER=0.19.8.1
export GETTEXT_VER_FILE=${BUILD_PREFIX}/.gettext_ver
export GETTEXT_ARGS="--enable-shared=no --enable-static --enable-fast-install --without-git"

export GLIB_BRANCH=2.52
export GLIB_VER=2.52.1
export GLIB_VER_FILE=${BUILD_PREFIX}/.glib_ver
export GLIB_ARGS="--disable-silent-rules --enable-shared=no --enable-static --enable-fast-install --disable-maintainer-mode --disable-dependency-tracking --disable-dtrace --disable-libelf"

export OPENSSH_VER=7.5p1
export OPENSSH_VER_FILE=${BUILD_PREFIX}/.openssh_ver
export OPENSSH_ARGS="--without-openssl --without-ssl-engine --with-sandbox=darwin"

export SSHFS_VER=2.9
export SSHFS_VER_FILE=${BUILD_PREFIX}/.sshfs_ver
export SSHFS_ARGS="--disable-dependency-tracking"
export SSHFS_CFLAGS="-D_FILE_OFFSET_BITS=64 -I${BUILD_PREFIX}/include/glib-2.0 -I${BUILD_PREFIX}/lib/glib-2.0/include -I/usr/local/include/osxfuse -I/usr/local/include/osxfuse/fuse"
export SSHFS_LIBS="-framework Carbon -liconv -lintl -lglib-2.0 -lgthread-2.0 -losxfuse -L${BUILD_PREFIX}/lib/glib-2.0"


export LIBRESSL_VER=2.5.3
export LIBRESSL_VER_FILE=${BUILD_PREFIX}/.libressl_ver
export LIBRESSL_ARGS="--disable-dependency-tracking --enable-shared=no --enable-static"

export RSYNC_VER=3.1.2
export RSYNC_VER_FILE=${BUILD_PREFIX}/.rsync_ver
export RSYNC_ARGS="--with-included-popt --with-included-zlib"
export RSYNC_CFLAGS="-I${BUILD_PREFIX}/include/openssl"
export RSYNC_LIBS="-L${BUILD_PREFIX}/lib"

export FFI_VER=3.2.1
export FFI_VER_FILE=${BUILD_PREFIX}/.ffi_ver
export FFI_ARGS="--enable-shared=no --enable-static"
export LIBFFI_CFLAGS="-I${BUILD_PREFIX}/lib/libffi-${FFI_VER}/include"
export LIBFFI_LIBS="-L${BUILD_PREFIX}/lib -lffi"

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
export BUILD_ICONV=false
export BUILD_GETTEXT=false
export BUILD_GLIB=false
export BUILD_OPENSSH=false
export BUILD_RSYNC=false
export BUILD_SSHFS=false
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
        export BUILD_ICONV=true
        export BUILD_GETTEXT=true
        export BUILD_GLIB=true
        export BUILD_OPENSSH=false
        export BUILD_RSYNC=true
        export BUILD_SSHFS=true
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

if [ ! -f gettext-${GETTEXT_VER}.tar.gz ]; then
    echo "Downloading gettext-${GETTEXT_VER}.tar.gz"
    echo "From http://ftp.gnu.org/pub/gnu/gettext/gettext-${GETTEXT_VER}.tar.gz"
    curl -L http://ftp.gnu.org/pub/gnu/gettext/gettext-${GETTEXT_VER}.tar.gz -o gettext-${GETTEXT_VER}.tar.gz > /dev/null
fi

if [ ! -f libiconv-${ICONV_VER}.tar.gz ]; then
    echo "Downloading iconv-${ICONV_VER}.tar.gz"
    echo "From https://ftp.gnu.org/pub/gnu/libiconv/libiconv-${ICONV_VER}.tar.gz"
    curl -L https://ftp.gnu.org/pub/gnu/libiconv/libiconv-${ICONV_VER}.tar.gz -o libiconv-${ICONV_VER}.tar.gz > /dev/null
fi

if [ ! -f glib-${GLIB_VER}.tar.xz ]; then
    echo "Downloading glib-${GLIB_VER}.tar.xz"
    echo "From http://ftp.gnome.org/pub/GNOME/sources/glib/${GLIB_BRANCH}/glib-${GLIB_VER}.tar.xz"
    curl -L http://ftp.gnome.org/pub/GNOME/sources/glib/${GLIB_BRANCH}/glib-${GLIB_VER}.tar.xz -o glib-${GLIB_VER}.tar.xz > /dev/null
fi

if [ ! -f sshfs-${SSHFS_VER}.tar.gz ]; then
    echo "Downloading sshfs-${SSHFS_VER}.tar.gz"
    echo "From https://github.com/libfuse/sshfs/releases/download/sshfs-${SSHFS_VER}/sshfs-${SSHFS_VER}.tar.gz"
    curl -L https://github.com/libfuse/sshfs/releases/download/sshfs_${SSHFS_VER}/sshfs-${SSHFS_VER}.tar.gz -o sshfs-${SSHFS_VER}.tar.gz > /dev/null
    curl -L https://github.com/libfuse/sshfs/releases/download/sshfs_${SSHFS_VER}/sshfs-${SSHFS_VER}.tar.gz.asc -o sshfs-${SSHFS_VER}.tar.gz.asc > /dev/null
fi

if [ ! -f libressl-${LIBRESSL_VER}.tar.gz ]; then
    echo "Downloading libressl-${LIBRESSL_VER}.tar.gz"
    curl -L https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VER}.tar.gz -o libressl-${LIBRESSL_VER}.tar.gz > /dev/null
    curl -L https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VER}.tar.gz.asc -o libressl-${LIBRESSL_VER}.tar.gz.asc > /dev/null
fi

if [ ! -f openssh-${OPENSSH_VER}.tar.gz ]; then
    echo "Downloading openssh-${OPENSSH_VER}.tar.gz"
    curl -L https://mirrors.evowise.com/pub/OpenBSD/OpenSSH/portable/openssh-${OPENSSH_VER}.tar.gz -o openssh-${OPENSSH_VER}.tar.gz > /dev/null
    curl -L https://mirrors.evowise.com/pub/OpenBSD/OpenSSH/portable/openssh-${OPENSSH_VER}.tar.gz.asc -o openssh-${OPENSSH_VER}.tar.gz.asc > /dev/null
fi

if [ ! -f rsync-${RSYNC_VER}.tar.gz ]; then
    echo "Downloading rsync-${RSYNC_VER}.tar.gz"
    curl -L https://download.samba.org/pub/rsync/src/rsync-${RSYNC_VER}.tar.gz -o rsync-${RSYNC_VER}.tar.gz > /dev/null
    curl -L https://download.samba.org/pub/rsync/src/rsync-${RSYNC_VER}.tar.gz.asc -o rsync-${RSYNC_VER}.tar.gz.asc > /dev/null
fi
popd > /dev/null



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

if [ ! -f ${BUILD_PREFIX}/lib/libsodium.a ] || [ ! -f ${SODIUM_VER_FILE} ] || [ ! $(<${SODIUM_VER_FILE}) = ${SODIUM_VER} ]; then
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

if [ ! -f  ${BUILD_PREFIX}/lib/libiconv.a ] || [ ! -f ${ICONV_VER_FILE} ] || [ ! $(<${ICONV_VER_FILE}) = ${ICONV_VER} ]; then
    if [ ${BUILD_ICONV} = true ]; then
        echo "Building iconv ${ICONV_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/libiconv-${ICONV_VER}.tar.gz > /dev/null
        pushd libiconv-${ICONV_VER}
            ./configure --prefix=${BUILD_PREFIX} ${ICONV_ARGS} > /dev/null
            make > /dev/null
            make install > /dev/null
        popd
        rm -rf libiconv*
        echo ${ICONV_VER} > ${ICONV_VER_FILE}
    else
        echo "Not set to build iconv"
    fi
else
    echo "Not building iconv"
fi

if [ ! -f  ${BUILD_PREFIX}/lib/libintl.a ] || [ ! -f ${GETTEXT_VER_FILE} ] || [ ! $(<${GETTEXT_VER_FILE}) = ${GETTEXT_VER} ]; then
    if [ ${BUILD_GETTEXT} = true ]; then

        echo "Building gettext ${GETTEXT_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/gettext-${GETTEXT_VER}.tar.gz > /dev/null
        pushd gettext-${GETTEXT_VER}
            ./configure --prefix=${BUILD_PREFIX} ${GETTEXT_ARGS} > /dev/null
            make > /dev/null
            make install > /dev/null
        popd
        rm -rf gettext*
        echo ${GETTEXT_VER} > ${GETTEXT_VER_FILE}
    else
        echo "Not set to build gettext"
    fi
else
    echo "Not building gettext"
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

if [ ! -f ${BUILD_PREFIX}/lib/libglib-2.0.a ] || [ ! -f ${GLIB_VER_FILE} ] || [ ! $(<${GLIB_VER_FILE}) = ${GLIB_VER} ]; then
    if [ ${BUILD_GLIB} = true ]; then
        echo "Building glib ${GLIB_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/glib-${GLIB_VER}.tar.xz > /dev/null
        pushd glib-${GLIB_VER}
            PATH=${BUILD_PREFIX}/bin:${PATH} ./configure --prefix=${BUILD_PREFIX} ${GLIB_ARGS} > /dev/null
            PATH=${BUILD_PREFIX}/bin:${PATH} make > /dev/null
            PATH=${BUILD_PREFIX}/bin:${PATH} make install > /dev/null
        popd
        rm -rf glib*
        echo ${GLIB_VER} > ${GLIB_VER_FILE}
    else
        echo "Not set to build glib"
    fi
else
    echo "Not building glib"
fi

if [ ! -f ${BUILD_PREFIX}/bin/ssh-${OPENSSH_VER} ] || [ ! -f ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.ssh ] || [ ! -f ${OPENSSH_VER_FILE} ] || [ ! $(<${OPENSSH_VER_FILE}) = ${OPENSSH_VER} ]; then
    if [ ${BUILD_OPENSSH} = true ]; then
        echo "Building OpenSSH ${OPENSSH_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf openssh-*
        tar xf ../src/openssh-${OPENSSH_VER}.tar.gz > /dev/null
        pushd openssh-${OPENSSH_VER} > /dev/null
            patch < ../../always-askpass.patch
            ./configure --prefix=${BUILD_PREFIX} ${OPENSSH_ARGS} > /dev/null
            make install OPENSSL=no > /dev/null

            cp ssh ${BUILD_PREFIX}/bin/ssh-${OPENSSH_VER}
            cp ${BUILD_PREFIX}/bin/ssh-${OPENSSH_VER} ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.ssh
        popd > /dev/null
        rm -rf openssh*
        echo ${OPENSSH_VER} > ${OPENSSH_VER_FILE}
    else
        echo "Not set to build OpenSSH"
    fi
else
    echo "Not building OpenSSH"
fi

if [ ! -f ${BUILD_PREFIX}/bin/rsync-${RSYNC_VER} ] || [ ! -f ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.rsync ] || [ ! -f ${RSYNC_VER_FILE} ] || [ ! $(<${RSYNC_VER_FILE}) = ${RSYNC_VER} ]; then
    if [ ${BUILD_RSYNC} = true ]; then

        echo "Building Rsync ${RSYNC_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf rsync-*
        tar xf ../src/rsync-${RSYNC_VER}.tar.gz > /dev/null
        pushd rsync-${RSYNC_VER} > /dev/null
            ./configure --prefix=${BUILD_PREFIX} ${RSYNC_ARGS} > /dev/null
            make install > /dev/null

            cp rsync ${BUILD_PREFIX}/bin/rsync-${RSYNC_VER}
            cp ${BUILD_PREFIX}/bin/rsync-${RSYNC_VER} ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.rsync

        popd > /dev/null
        rm -rf rsync*
        echo ${RSYNC_VER} > ${RSYNC_VER_FILE}
    else
        echo "Not set to build Rsync"
    fi
else
    echo "Not building Rsync"
fi

if [ ! -f ${BUILD_PREFIX}/bin/sshfs-${SSHFS_VER} ] || [ ! -f ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.sshfs ] || [ ! -f ${SSHFS_VER_FILE} ] || [ ! $(<${SSHFS_VER_FILE}) = ${SSHFS_VER} ]; then
    if [ ${BUILD_SSHFS} = true ]; then

        echo "Building SSHFS ${SSHFS_VER} for ${TARGET} in ${BUILD_PREFIX}"

        tar xf ../src/sshfs-${SSHFS_VER}.tar.gz > /dev/null
        pushd sshfs-${SSHFS_VER}
            ./configure --prefix=${BUILD_PREFIX} ${SSHFS_ARGS} > /dev/null
            make > /dev/null
            echo "Building static SSHFS ${SSHFS_VER} for ${TARGET} in ${BUILD_PREFIX}"
            cp sshfs ${BUILD_PREFIX}/bin/sshfs-${SSHFS_VER}
            cp ${BUILD_PREFIX}/bin/sshfs-${SSHFS_VER} ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.sshfs
        popd
        rm -rf sshfs*
        echo ${SSHFS_VER} > ${SSHFS_VER_FILE}
    else
        echo "Not set to build sshfs"
    fi
else
    echo "Not building sshfs"
fi

popd > /dev/null
