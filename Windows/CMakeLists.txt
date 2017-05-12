cmake_minimum_required(VERSION 3.8)
project(SafeDriveSDK)

set(CMAKE_SUPPRESS_REGENERATION true)

set(CMAKE_CXX_STANDARD 17)

get_filename_component(PARENT_DIR ${CMAKE_CURRENT_SOURCE_DIR} DIRECTORY)
set(DEP_DIR "${PARENT_DIR}/dep/${TARGET}/${TOOLSET}/${CONFIGURATION}/")
set(DIST_DIR "${PARENT_DIR}/dist/${TARGET}/${TOOLSET}/${CONFIGURATION}/")
set(BUILD_DIR "${PARENT_DIR}/build/${TARGET}/${TOOLSET}/${CONFIGURATION}/")

if(CMAKE_SYSTEM_NAME MATCHES "Darwin")
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Linux")
endif()


if(CMAKE_HOST_WIN32)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /std:c++latest")
    set(PLATFORM_SOURCES
            stdafx.cpp
            stdafx.h
            targetver.h
            dllmain.cpp)
endif()

set(HEADER_FILES ../dist/$ENV{TARGET}/include/sddk.h)
set(LIBRARY_OUTPUT_PATH "${DIST_DIR}/lib")

include_directories("${DEP_DIR}/include")
include_directories("${DIST_DIR}/include")
link_directories("${DEP_DIR}/lib")
link_directories("${DIST_DIR}/lib")

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

