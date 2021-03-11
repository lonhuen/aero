cmake_minimum_required (VERSION 3.5)
project (quail)

set (CMAKE_CXX_STANDARD 17)


include_directories (./src)
file (GLOB_RECURSE SourceFiles src/*.cpp)
add_executable (quail ${SourceFiles})

find_package (glog REQUIRED)
target_link_libraries (quail glog::glog)

find_package (SEAL 3.6 REQUIRED)
target_link_libraries (quail SEAL::seal)

