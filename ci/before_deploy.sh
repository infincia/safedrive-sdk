#!/usr/bin/env bash

# `before_deploy` phase: here we package the build artifacts

set -ex

. $(dirname $0)/utils.sh


export DIST_PREFIX=${PWD}/target/${TARGET}/release

mk_tarball() {
    # release tarball will look like 'rust-everywhere-v1.2.3-x86_64-unknown-linux-gnu.tar.gz'
    TAG=$(git describe)
    pushd ${DIST_PREFIX}
    tar -zcf safedrive-cli-${TAG}-${TARGET}.tar.gz safedrive
    popd
}

# Package your artifacts in a .deb file
# NOTE right now you can only package binaries using the `dobin` command. Simply call
# `dobin [file..]` to include one or more binaries in your .deb package. I'll add more commands to
# install other things like manpages (`doman`) as the needs arise.
# XXX This .deb packaging is minimal -- just to make your app installable via `dpkg` -- and doesn't
# fully conform to Debian packaging guideliens (`lintian` raises a few warnings/errors)
mk_deb() {
    # TODO update this part to package the artifacts that make sense for your project
    dobin ${DIST_PREFIX}/io.safedrive.SafeDrive.cli
    case ${TARGET} in
        i686-unknown-linux-musl|x86_64-unknown-linux-musl)
            ;;
        *)
            dolib dist/${TARGET}/lib/libsddk.a
            doinclude include/sddk.h
            ;;
    esac

}

main() {
    mk_tarball

    if [ ${TRAVIS_OS_NAME} = linux ]; then
        if [ ! -z ${MAKE_DEB} ]; then
            dtd=$(mktempd)
            mkdir -p $dtd/debian/usr/bin
            mkdir -p $dtd/debian/usr/lib
            mkdir -p $dtd/debian/usr/include

            mk_deb

            TAG=$(git describe)


            mkdir -p $dtd/debian/DEBIAN
            cat >$dtd/debian/DEBIAN/control <<EOF
Package: ${PROJECT_NAME}
Version: ${TAG#v}
Architecture: $(architecture ${TARGET})
Maintainer: ${DEB_MAINTAINER}
Description: ${DEB_DESCRIPTION}
EOF

            fakeroot dpkg-deb --build $dtd/debian
            mv $dtd/debian.deb ${PROJECT_NAME}-${TAG}-${TARGET}.deb
            rm -r $dtd
        fi
    fi
}

main
