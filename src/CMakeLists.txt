#Compile and add the rust lib to the tree with name "ealovega::serialcxx"
add_library_rust(PATH serialcxx NAMESPACE ealovega)

#Add lto support
corrosion_add_target_rustflags(serialcxx_cxxbridge -Clinker-plugin-lto)

set_property(TARGET serialcxx_cxxbridge 
             PROPERTY Rust_TOOLCHAIN "stable")

if ("${CMAKE_SYSTEM_NAME}" STREQUAL "Windows")

elseif ("${CMAKE_SYSTEM_NAME}" STREQUAL "Linux")
    #Include the udev include files and shared lib, needed for port enumeration on linux.
    include(../cmake/findUdev.cmake)
    target_include_directories(serialcxx-total
            INTERFACE
            ${UDEV_INCLUDE_DIRS}
            )

    target_link_libraries(serialcxx-total
            INTERFACE
            ${UDEV_LIBRARIES}
            )
endif ()

#TODO ignore this last part unless a test flag is set, the above exposes the library to others in the tree already.

#Tests CPP compatibility, needs to be removed in final release
add_executable(${PROJECT_NAME}-test)

target_sources(${PROJECT_NAME}-test
        PRIVATE
        main.cpp
        )

target_link_libraries(${PROJECT_NAME}-test
        PUBLIC
        ealovega::serialcxx
        )

include(CheckIPOSupported)
check_ipo_supported()
set_target_properties(${PROJECT_NAME}-test PROPERTIES INTERPROCEDURAL_OPTIMIZATION TRUE)
