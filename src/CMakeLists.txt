#Gets udev for port enumeration
include(../cmake/findUdev.cmake)

#Compile and add the rust lib to the tree with name "ealovega::serialcxx"
add_library_rust(PATH serialcxx NAMESPACE ealovega)

#TODO add a windows equivelent to udev include to allow for this library to be used on windows.
#Rn it linker errors due to this not being included. just if else over windows and linux.

#Include the udev include files and shared lib, needed for port enumeration on linux.
target_include_directories(serialcxx-total
        INTERFACE
        ${UDEV_INCLUDE_DIRS}
        )

target_link_libraries(serialcxx-total
        INTERFACE
        ${UDEV_LIBRARIES}
        )

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
