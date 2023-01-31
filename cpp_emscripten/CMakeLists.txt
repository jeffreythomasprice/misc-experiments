cmake_minimum_required(VERSION 3.12)

project(emscripten_experiment)

file(GLOB_RECURSE sources "*.cpp" "*.h")
add_executable(client ${sources})
set_property(TARGET client PROPERTY CXX_STANDARD 20)
target_link_options(client PRIVATE "--bind")

add_custom_command(
	OUTPUT "${CMAKE_BINARY_DIR}/.notARealFile"
	COMMAND ${CMAKE_SOURCE_DIR}/scripts/post-build.sh "${CMAKE_SOURCE_DIR}" "${CMAKE_BINARY_DIR}"
)
add_custom_target(clientPostBuild DEPENDS "${CMAKE_BINARY_DIR}/.notARealFile")
add_dependencies(client clientPostBuild)

add_custom_target(
	run
	"${CMAKE_SOURCE_DIR}/scripts/run.sh" "${CMAKE_SOURCE_DIR}"
	DEPENDS client
	WORKING_DIRECTORY "${CMAKE_BINARY_DIR}"
	VERBATIM
)