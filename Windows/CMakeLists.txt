cmake_minimum_required(VERSION 3.8)
project(SafeDriveSDK)

set(CMAKE_CXX_STANDARD 17)

if(NOT DEFINED ENV{TARGET})
    set(ENV{TARGET} x86_64-apple-darwin)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Darwin")
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Linux")
endif()


if(CMAKE_HOST_WIN32)
    set(PLATFORM_SOURCES
            stdafx.cpp
            stdafx.h
            targetver.h
            dllmain.cpp)
endif()

set(LIBRARY_OUTPUT_PATH ../dist/$ENV{TARGET}/lib/)

include_directories(../dist/$ENV{TARGET}/include)
link_directories("../dist/$ENV{TARGET}/lib")

set(HEADER_FILES ../dist/$ENV{TARGET}/include/sddk.h)


set(SOURCE_FILES
        AccountDetails.cpp
        AccountStatus.cpp
        Folder.cpp
        SafeDriveSDK.cpp
        SafeDriveSDK.h
        SDKException.cpp
        SoftwareClient.cpp
        SyncSession.cpp
        )


add_library(SafeDriveSDK ${HEADER_FILES} ${PLATFORM_SOURCES} ${SOURCE_FILES} test.cpp test.h)

if(CMAKE_HOST_WIN32)
    link_libraries(SafeDriveSDK sddk.lib)
else()
    link_libraries(SafeDriveSDK sddk.a)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Darwin")
    target_link_libraries(SafeDriveSDK z "-framework Foundation" "-framework Security" objc)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Linux")
    target_link_libraries(SafeDriveSDK z)
endif()


add_executable(SafeDriveSDKTest SafeDriveSDK.h test.cpp test.h)
add_dependencies(SafeDriveSDKTest SafeDriveSDK)
target_link_libraries(SafeDriveSDKTest SafeDriveSDK)

