cmake_minimum_required(VERSION 3.8)
project(SafeDriveSDK)

set(CMAKE_SUPPRESS_REGENERATION true)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_OSX_DEPLOYMENT_TARGET "10.9")


get_filename_component(PARENT_DIR ${CMAKE_CURRENT_SOURCE_DIR} DIRECTORY)

if(CMAKE_SYSTEM_NAME MATCHES "Windows")
    add_definitions(-DUNICODE -D_UNICODE)
    #set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /std:c++latest")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}")
    if(${CONFIGURATION} MATCHES "ReleaseDLL")
        set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} /MD")
        set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} /MDd")

        set(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} /MD")
        set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} /MDd")
    else()
        set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} /MT")
        set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} /MTd")

        set(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} /MT")
        set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} /MTd")
    endif()
    set(DEP_DIR "${PARENT_DIR}/dep/${TARGET}/${TOOLSET}/${CONFIGURATION}/")
    set(DIST_DIR "${PARENT_DIR}/dist/${TARGET}/${TOOLSET}/${CONFIGURATION}/")
    set(BUILD_DIR "${PARENT_DIR}/build/${PLATFORM}/${CONFIGURATION}/${TOOLSET}/")
    set(PLATFORM_SOURCES
            stdafx.h
            targetver.h)
else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}" "${CFLAGS}")
    set(CMAKE_CPP_FLAGS "${CMAKE_CPP_FLAGS}" "${CPPFLAGS}")

    set(DEP_DIR "${PARENT_DIR}/dep/${TARGET}/")
    set(DIST_DIR "${PARENT_DIR}/dist/${TARGET}/")
    set(BUILD_DIR "${PARENT_DIR}/build/${TARGET}/")
endif()

include_directories("${DEP_DIR}/include")
include_directories("${DIST_DIR}/include")
include_directories(".")
link_directories("${DEP_DIR}")
link_directories("${DIST_DIR}/lib")

set(SOURCE_FILES
        SafeDriveSDK.cpp
        SafeDriveSDK.h
        )

set(HEADER_FILES
    SafeDriveSDK.h
    )



add_library(SafeDriveSDK ${PLATFORM_SOURCES} ${SOURCE_FILES})

if(CMAKE_SYSTEM_NAME MATCHES "Windows")
    target_link_libraries(SafeDriveSDK)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Darwin")
    target_link_libraries(SafeDriveSDK)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Linux")
    target_link_libraries(SafeDriveSDK)
endif()


add_executable(SafeDriveSDKTest test.cpp)
add_dependencies(SafeDriveSDKTest SafeDriveSDK)
if(CMAKE_SYSTEM_NAME MATCHES "Windows")
    target_link_libraries(SafeDriveSDKTest sddk.lib user32.lib secur32.lib userenv.lib ncrypt.lib crypt32.lib sodium.lib ws2_32.lib SafeDriveSDK)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Darwin")
    target_link_libraries(SafeDriveSDKTest z "-framework Foundation" "-framework Security" resolv objc sddk.a SafeDriveSDK)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Linux")
    target_link_libraries(SafeDriveSDKTest z sddk.a SafeDriveSDK)
endif()

