cmake_minimum_required(VERSION 3.5)

find_package(PkgConfig REQUIRED)
pkg_check_modules(FUSE REQUIRED fuse)

file(GLOB_RECURSE sources src/*.cpp src/*.h)

project(fusehelper)
add_library(fusehelper SHARED ${sources})
target_link_libraries(fusehelper ${FUSE_LIBRARIES})
target_include_directories(fusehelper PUBLIC ${FUSE_INCLUDE_DIRS})
target_compile_options(fusehelper PUBLIC ${FUSE_CFLAGS_OTHER})