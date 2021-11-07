add_executable(${PROJECT_NAME}-test)

target_sources(${PROJECT_NAME}-test
    PRIVATE
        main.cpp
)

add_library_rust(PATH serialcxx NAMESPACE ealovega)

target_link_libraries(${PROJECT_NAME}-test
    PUBLIC
        ealovega::serialcxx
)
