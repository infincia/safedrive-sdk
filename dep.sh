#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

export SRC_PREFIX=${PWD}/src
export PATCH_PREFIX=${PWD}/builder/patches
export STATIC_DEP_PREFIX=${PWD}/dep-static

export DIST_PREFIX=${PWD}/target/${TARGET}/release
export BUILD_PREFIX=${DIST_PREFIX}/deps


mkdir -p ${BUILD_PREFIX}/lib
mkdir -p ${BUILD_PREFIX}/include
mkdir -p ${BUILD_PREFIX}/bin
mkdir -p ${BUILD_PREFIX}/share


mkdir -p ${DIST_PREFIX}/lib
mkdir -p ${DIST_PREFIX}/include
mkdir -p ${DIST_PREFIX}/bin

mkdir -p src
mkdir -p build

clear_man() {
    pushd ${BUILD_PREFIX}/share
    rm -rf man
    popd
}

# these are at the top for visibility, changing a version will always cause a rebuild, otherwise
# they will only be rebuilt if the built product is not found
export SODIUM_VER=1.0.14
export SODIUM_VER_FILE=${BUILD_PREFIX}/.sodium_ver
export SODIUM_ARGS="--enable-shared=no"

export LIBDBUS_VER=1.12.2
export LIBDBUS_VER_FILE=${BUILD_PREFIX}/.dbus_ver
export LIBDBUS_ARGS="--enable-shared=no --disable-tests --with-x=no --disable-systemd --disable-launchd --disable-libaudit --disable-selinux --disable-apparmor"

export EXPAT_VER=2.2.5
export EXPAT_VER_FILE=${BUILD_PREFIX}/.expat_ver
export EXPAT_ARGS="--enable-shared=no"

export OPENSSH_VER=7.6p1
export OPENSSH_VER_FILE=${BUILD_PREFIX}/.openssh_ver
export OPENSSH_ARGS="--without-openssl --without-ssl-engine --with-sandbox=darwin"

export LIBRESSL_VER=2.6.3
export LIBRESSL_VER_FILE=${BUILD_PREFIX}/.libressl_ver
export LIBRESSL_ARGS="--disable-dependency-tracking --enable-shared=no"

export RSYNC_VER=3.1.2
export RSYNC_VER_FILE=${BUILD_PREFIX}/.rsync_ver
export RSYNC_ARGS="--with-included-popt --with-included-zlib"
export RSYNC_CFLAGS="-I${BUILD_PREFIX}/include/openssl"
export RSYNC_LIBS="-L${BUILD_PREFIX}/lib"

export LIBSSH2_VER=1.8.0
export LIBSSH2_VER_FILE=${BUILD_PREFIX}/.libssh2_ver
export LIBSSH2_ARGS="--enable-shared=no --with-libssl-prefix=${BUILD_PREFIX}"

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
export BUILD_OPENSSH=false
export BUILD_RSYNC=false
export BUILD_LIBRESSL=false
export BUILD_LIBSSH2=false

export CFLAGS="-fPIC -O2 -g -I${BUILD_PREFIX}/include"
export CPPFLAGS="-fPIC -O2 -g -I${BUILD_PREFIX}/include"
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
        export BUILD_OPENSSH=true
        export BUILD_RSYNC=true
        export BUILD_LIBRESSL=true
        export BUILD_LIBSSH2=true
        ;;
    x86_64-unknown-linux-gnu)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        export BUILD_EXPAT=false
        export BUILD_DBUS=false
        export BUILD_LIBRESSL=false
        export BUILD_LIBSSH2=false
        ;;
    x86_64-unknown-linux-musl)
        export CFLAGS="${CFLAGS} -I/usr/include"
        export CPPFLAGS="${CPPFLAGS} -I/usr/include"
        export LDFLAGS="${LDFLAGS}"
        export CC=musl-gcc
        export BUILD_EXPAT=true
        export BUILD_DBUS=true
        export BUILD_LIBRESSL=true
        export BUILD_LIBSSH2=true
        export BUILD_OPENSSH=true
        export BUILD_RSYNC=true
        ;;
    wasm32-unknown-emscripten)
        export SODIUM_ARGS="${SODIUM_ARGS}"
        export CONFIGURE_PREFIX="emconfigure"
        export TOTAL_MEMORY=16777216
        export WASM_LDFLAGS="-s RESERVED_FUNCTION_POINTERS=8 -s NO_DYNAMIC_EXECUTION=1 -s RUNNING_JS_OPTS=1 -s OUTLINING_LIMIT=20000 -O3 -s ALLOW_MEMORY_GROWTH=1"
        export CFLAGS="-Os"
        export CC=emcc
        export PKG_CONFIG_ALLOW_CROSS=1
        export EXPORTED_LIBSODIUM_FUNCTIONS='["_crypto_aead_chacha20poly1305_abytes","_crypto_aead_chacha20poly1305_decrypt","_crypto_aead_chacha20poly1305_decrypt_detached","_crypto_aead_chacha20poly1305_encrypt","_crypto_aead_chacha20poly1305_encrypt_detached","_crypto_aead_chacha20poly1305_ietf_abytes","_crypto_aead_chacha20poly1305_ietf_decrypt","_crypto_aead_chacha20poly1305_ietf_decrypt_detached","_crypto_aead_chacha20poly1305_ietf_encrypt","_crypto_aead_chacha20poly1305_ietf_encrypt_detached","_crypto_aead_chacha20poly1305_ietf_keybytes","_crypto_aead_chacha20poly1305_ietf_keygen","_crypto_aead_chacha20poly1305_ietf_npubbytes","_crypto_aead_chacha20poly1305_ietf_nsecbytes","_crypto_aead_chacha20poly1305_keybytes","_crypto_aead_chacha20poly1305_keygen","_crypto_aead_chacha20poly1305_npubbytes","_crypto_aead_chacha20poly1305_nsecbytes","_crypto_aead_xchacha20poly1305_ietf_abytes","_crypto_aead_xchacha20poly1305_ietf_decrypt","_crypto_aead_xchacha20poly1305_ietf_decrypt_detached","_crypto_aead_xchacha20poly1305_ietf_encrypt","_crypto_aead_xchacha20poly1305_ietf_encrypt_detached","_crypto_aead_xchacha20poly1305_ietf_keybytes","_crypto_aead_xchacha20poly1305_ietf_keygen","_crypto_aead_xchacha20poly1305_ietf_npubbytes","_crypto_aead_xchacha20poly1305_ietf_nsecbytes","_crypto_auth","_crypto_auth_bytes","_crypto_auth_keybytes","_crypto_auth_keygen","_crypto_auth_verify","_crypto_box_beforenm","_crypto_box_beforenmbytes","_crypto_box_detached","_crypto_box_detached_afternm","_crypto_box_easy","_crypto_box_easy_afternm","_crypto_box_keypair","_crypto_box_macbytes","_crypto_box_noncebytes","_crypto_box_open_detached","_crypto_box_open_detached_afternm","_crypto_box_open_easy","_crypto_box_open_easy_afternm","_crypto_box_publickeybytes","_crypto_box_seal","_crypto_box_seal_open","_crypto_box_sealbytes","_crypto_box_secretkeybytes","_crypto_box_seed_keypair","_crypto_box_seedbytes","_crypto_core_hchacha20","_crypto_core_hchacha20_constbytes","_crypto_core_hchacha20_inputbytes","_crypto_core_hchacha20_keybytes","_crypto_core_hchacha20_outputbytes","_crypto_generichash","_crypto_generichash_bytes","_crypto_generichash_bytes_max","_crypto_generichash_bytes_min","_crypto_generichash_final","_crypto_generichash_init","_crypto_generichash_keybytes","_crypto_generichash_keybytes_max","_crypto_generichash_keybytes_min","_crypto_generichash_keygen","_crypto_generichash_statebytes","_crypto_generichash_update","_crypto_hash","_crypto_hash_bytes","_crypto_kdf_bytes_max","_crypto_kdf_bytes_min","_crypto_kdf_contextbytes","_crypto_kdf_derive_from_key","_crypto_kdf_keybytes","_crypto_kdf_keygen","_crypto_kx_client_session_keys","_crypto_kx_keypair","_crypto_kx_publickeybytes","_crypto_kx_secretkeybytes","_crypto_kx_seed_keypair","_crypto_kx_seedbytes","_crypto_kx_server_session_keys","_crypto_kx_sessionkeybytes","_crypto_pwhash_bytes_max","_crypto_pwhash_bytes_min","_crypto_pwhash_memlimit_max","_crypto_pwhash_memlimit_min","_crypto_pwhash_opslimit_max","_crypto_pwhash_opslimit_min","_crypto_pwhash_passwd_max","_crypto_pwhash_passwd_min","_crypto_scalarmult","_crypto_scalarmult_base","_crypto_scalarmult_bytes","_crypto_scalarmult_scalarbytes","_crypto_secretbox_detached","_crypto_secretbox_easy","_crypto_secretbox_keybytes","_crypto_secretbox_keygen","_crypto_secretbox_macbytes","_crypto_secretbox_noncebytes","_crypto_secretbox_open_detached","_crypto_secretbox_open_easy","_crypto_shorthash","_crypto_shorthash_bytes","_crypto_shorthash_keybytes","_crypto_shorthash_keygen","_crypto_sign","_crypto_sign_bytes","_crypto_sign_detached","_crypto_sign_ed25519_pk_to_curve25519","_crypto_sign_ed25519_sk_to_curve25519","_crypto_sign_final_create","_crypto_sign_final_verify","_crypto_sign_init","_crypto_sign_keypair","_crypto_sign_open","_crypto_sign_publickeybytes","_crypto_sign_secretkeybytes","_crypto_sign_seed_keypair","_crypto_sign_seedbytes","_crypto_sign_statebytes","_crypto_sign_update","_crypto_sign_verify_detached","_crypto_stream_keygen","_randombytes","_randombytes_buf","_randombytes_buf_deterministic","_randombytes_close","_randombytes_random","_randombytes_seedbytes","_randombytes_stir","_randombytes_uniform","_sodium_bin2hex","_sodium_hex2bin","_sodium_init","_sodium_library_minimal","_sodium_library_version_major","_sodium_library_version_minor","_sodium_version_string"]'
        export JS_EXPORTS_FLAGS="-s EXPORTED_FUNCTIONS=${EXPORTED_LIBSODIUM_FUNCTIONS}"
        ;;
    *)
        ;;
esac




# grab sources

pushd ${SRC_PREFIX} > /dev/null

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

if [ ! -f openssh-${OPENSSH_VER}.tar.gz ]; then
    echo "Downloading openssh-${OPENSSH_VER}.tar.gz"
    curl -L https://mirrors.evowise.com/pub/OpenBSD/OpenSSH/portable/openssh-${OPENSSH_VER}.tar.gz -o openssh-${OPENSSH_VER}.tar.gz > /dev/null
    curl -L https://mirrors.evowise.com/pub/OpenBSD/OpenSSH/portable/openssh-${OPENSSH_VER}.tar.gz.asc -o openssh-${OPENSSH_VER}.tar.gz.asc > /dev/null
fi

if [ ! -f rsync-${RSYNC_VER}.tar.gz ] || [ ! -f rsync-patches-${RSYNC_VER}.tar.gz ]; then
    echo "Downloading rsync-${RSYNC_VER}.tar.gz"
    curl -L https://download.samba.org/pub/rsync/src/rsync-${RSYNC_VER}.tar.gz -o rsync-${RSYNC_VER}.tar.gz > /dev/null
    curl -L https://download.samba.org/pub/rsync/src/rsync-${RSYNC_VER}.tar.gz.asc -o rsync-${RSYNC_VER}.tar.gz.asc > /dev/null
    curl -L https://download.samba.org/pub/rsync/src/rsync-patches-${RSYNC_VER}.tar.gz -o rsync-patches-${RSYNC_VER}.tar.gz > /dev/null
    curl -L https://download.samba.org/pub/rsync/src/rsync-patches-${RSYNC_VER}.tar.gz.asc -o rsync-patches-${RSYNC_VER}.tar.gz.asc > /dev/null
fi


if [ ! -f libssh2-${LIBSSH2_VER}.tar.gz ]; then
    echo "Downloading libssh2-${LIBSSH2_VER}.tar.gz"
    curl -L https://www.libssh2.org/download/libssh2-${LIBSSH2_VER}.tar.gz -o libssh2-${LIBSSH2_VER}.tar.gz > /dev/null
    curl -L https://www.libssh2.org/download/libssh2-${LIBSSH2_VER}.tar.gz.asc -o libssh2-${LIBSSH2_VER}.tar.gz.asc > /dev/null
fi
popd > /dev/null



pushd ${BUILD_PREFIX} > /dev/null

if [ ${BUILD_LIBRESSL} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/lib/libssl.a ] || [ ! -f ${LIBRESSL_VER_FILE} ] || [ ! $(<${LIBRESSL_VER_FILE}) = ${LIBRESSL_VER} ]; then

        echo "Building LibreSSL ${LIBRESSL_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf libressl*
        clear_man
        tar xf ${SRC_PREFIX}/libressl-${LIBRESSL_VER}.tar.gz > /dev/null
        pushd libressl-${LIBRESSL_VER} > /dev/null
            case ${TARGET} in
                i686-unknown-linux-musl|x86_64-unknown-linux-musl)
                    patch -p0 < ${PATCH_PREFIX}/libressl-musl.patch
                    ;;
                *)
                    ;;
            esac
            ./configure --prefix=${BUILD_PREFIX} ${LIBRESSL_ARGS} > /dev/null
            make install > /dev/null
        popd > /dev/null
        rm -rf libressl*
        echo ${LIBRESSL_VER} > ${LIBRESSL_VER_FILE}
    else
        echo "Not building LibreSSL"
    fi
else
    echo "Not set to build LibreSSL"
fi

if [ ${BUILD_LIBSSH2} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/lib/libssh2.a ] || [ ! -f ${LIBSSH2_VER_FILE} ] || [ ! $(<${LIBSSH2_VER_FILE}) = ${LIBSSH2_VER} ]; then
        echo "Building libssh2 ${LIBSSH2_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf libssh2-*
        clear_man
        tar xf ${SRC_PREFIX}/libssh2-${LIBSSH2_VER}.tar.gz > /dev/null
        pushd libssh2-${LIBSSH2_VER} > /dev/null
        ./configure --prefix=${BUILD_PREFIX} ${LIBSSH2_ARGS} > /dev/null
        make install > /dev/null
        popd > /dev/null
        rm -rf libssh2*
        echo ${LIBSSH2_VER} > ${LIBSSH2_VER_FILE}
    else
        echo "Not building libssh2"
    fi
else
    echo "Not set to build libssh2"
fi

if [ ${BUILD_EXPAT} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/lib/libexpat.a ] || [ ! -f ${EXPAT_VER_FILE} ] || [ ! $(<${EXPAT_VER_FILE}) = ${EXPAT_VER} ]; then

        echo "Building libexpat ${EXPAT_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf expat*
        clear_man
        tar xf ${SRC_PREFIX}/expat-${EXPAT_VER}.tar.bz2 > /dev/null
        pushd expat-${EXPAT_VER}
        ./configure --prefix=${BUILD_PREFIX} ${EXPAT_ARGS} > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf expat*
        echo ${EXPAT_VER} > ${EXPAT_VER_FILE}
    else
        echo "Not building libexpat"
    fi
else
    echo "Not set to build libexpat"
fi

if [ ${BUILD_DBUS} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/lib/libdbus-1.a ] || [ ! -f ${LIBDBUS_VER_FILE} ] || [ ! $(<${LIBDBUS_VER_FILE}) = ${LIBDBUS_VER} ]; then
        echo "Building libdbus ${LIBDBUS_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf dbus*
        clear_man
        tar xf ${SRC_PREFIX}/dbus-${LIBDBUS_VER}.tar.gz > /dev/null
        pushd dbus-${LIBDBUS_VER}
        ./configure --prefix=${BUILD_PREFIX} ${LIBDBUS_ARGS} > /dev/null
        make > /dev/null
        make install > /dev/null
        popd
        rm -rf dbus*
        echo ${LIBDBUS_VER} > ${LIBDBUS_VER_FILE}
    else
        echo "Not building libdbus"
    fi
else
    echo "Not set to build libdbus"
fi

if [ ${BUILD_LIBSODIUM} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/lib/libsodium.a ] || [ ! -f ${SODIUM_VER_FILE} ] || [ ! $(<${SODIUM_VER_FILE}) = ${SODIUM_VER} ]; then

        echo "Building libsodium ${SODIUM_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf libsodium*
        clear_man
        tar xf ${SRC_PREFIX}/libsodium-${SODIUM_VER}.tar.gz > /dev/null
        pushd libsodium-${SODIUM_VER}
        ${CONFIGURE_PREFIX} ./configure --prefix=${BUILD_PREFIX} ${SODIUM_ARGS} > /dev/null
        case ${TARGET} in
            wasm32-unknown-emscripten)
                emmake make -j4 install-exec install-data V=1
                emcc -s BINARYEN=1 "${CFLAGS}" --llvm-lto 1 --memory-init-file 0 ${CPPFLAGS} ${JS_EXPORTS_FLAGS} "${BUILD_PREFIX}/lib/libsodium.a" -o "${BUILD_PREFIX}/lib/libsodium.js"
                ;;
            *)
                make clean > /dev/null
                make > /dev/null
                make install > /dev/null
                ;;
        esac
        popd
        rm -rf libsodium*
        echo ${SODIUM_VER} > ${SODIUM_VER_FILE}

    else
        echo "Not building libsodium"
    fi
else
    echo "Not set to build libsodium"
fi

if [ ${BUILD_OPENSSH} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/bin/ssh-${OPENSSH_VER} ] || [ ! -f ${OPENSSH_VER_FILE} ] || [ ! $(<${OPENSSH_VER_FILE}) = ${OPENSSH_VER} ]; then
        echo "Building OpenSSH ${OPENSSH_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf openssh-*
        clear_man
        tar xf ${SRC_PREFIX}/openssh-${OPENSSH_VER}.tar.gz > /dev/null
        pushd openssh-${OPENSSH_VER} > /dev/null
            patch < ${PATCH_PREFIX}/always-askpass.patch
            ./configure --prefix=${BUILD_PREFIX} ${OPENSSH_ARGS} > /dev/null
            make install OPENSSL=no > /dev/null

            cp ssh ${BUILD_PREFIX}/bin/ssh-${OPENSSH_VER}
        popd > /dev/null
        rm -rf openssh*
        echo ${OPENSSH_VER} > ${OPENSSH_VER_FILE}
    fi
    cp ${BUILD_PREFIX}/bin/ssh-${OPENSSH_VER} ${DIST_PREFIX}/io.safedrive.SafeDrive.ssh
else
    echo "Not set to build OpenSSH"
fi


if [ ${BUILD_RSYNC} = true ]; then
    if [ ! -f ${BUILD_PREFIX}/bin/rsync-${RSYNC_VER} ] || [ ! -f ${RSYNC_VER_FILE} ] || [ ! $(<${RSYNC_VER_FILE}) = ${RSYNC_VER} ]; then
        echo "Building Rsync ${RSYNC_VER} for ${TARGET} in ${BUILD_PREFIX}"
        rm -rf rsync-*
        clear_man
        tar xf ${SRC_PREFIX}/rsync-${RSYNC_VER}.tar.gz > /dev/null
        tar xf ${SRC_PREFIX}/rsync-patches-${RSYNC_VER}.tar.gz > /dev/null
        pushd rsync-${RSYNC_VER} > /dev/null
            patch -p1 < patches/fileflags.diff
            patch -p1 < patches/crtimes.diff
            ./configure --prefix=${BUILD_PREFIX} ${RSYNC_ARGS} > /dev/null
            make install > /dev/null
            cp rsync ${BUILD_PREFIX}/bin/rsync-${RSYNC_VER}
        popd > /dev/null
        rm -rf rsync*
        echo ${RSYNC_VER} > ${RSYNC_VER_FILE}
    fi
    cp ${BUILD_PREFIX}/bin/rsync-${RSYNC_VER} ${DIST_PREFIX}/io.safedrive.SafeDrive.rsync
else
    echo "Not set to build Rsync"
fi

popd > /dev/null
