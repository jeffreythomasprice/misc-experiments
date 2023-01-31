cmake_minimum_required(VERSION 3.12)

project(emscripten_experiment)

file(GLOB_RECURSE sources "*.cpp" "*.h")
add_executable(client ${sources})
set_property(TARGET client PROPERTY CXX_STANDARD 20)

add_custom_command(
	OUTPUT "${CMAKE_BINARY_DIR}/.notARealFile"
	COMMAND ${CMAKE_SOURCE_DIR}/post-build.sh "${CMAKE_SOURCE_DIR}" "${CMAKE_BINARY_DIR}"
)
add_custom_target(clientPostBuild DEPENDS "${CMAKE_BINARY_DIR}/.notARealFile")
add_dependencies(client clientPostBuild)
