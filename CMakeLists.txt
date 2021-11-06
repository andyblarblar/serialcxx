cmake_minimum_required(VERSION 3.16)

set(CMAKE_EXPORT_COMPILE_COMMANDS true)

project(serialcxx CXX)

include(FetchContent)

#TODO check if we can define an arm version for raspis
if("${CMAKE_SYSTEM_NAME}" STREQUAL "Windows")
    set(Rust_CARGO_TARGET "x86_64-pc-windows-msvc") #TODO This fails under debug in msvc
elseif("${CMAKE_SYSTEM_NAME}" STREQUAL "Linux")
    set(Rust_CARGO_TARGET "x86_64-unknown-linux-gnu")
else()
    message(FATAL_ERROR "hardcoded platformchecks not supported outside windows-gnu and linux-gnu")
endif()

#Corrosion is used to add the rust target to the cmake dependency tree.
FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/AndrewGaspar/corrosion.git
        GIT_TAG origin/master
)
FetchContent_MakeAvailable(Corrosion)

include(cmake/corrosion_cxx.cmake)

add_subdirectory(src)
