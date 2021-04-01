cmake_minimum_required (VERSION 3.5)
project (quail)

set (CMAKE_CXX_STANDARD 17)

# list(APPEND CMAKE_PREFIX_PATH "/home/richa/packages/vcpkg/packages/seal_x64-linux/share/seal/")
# list(APPEND CMAKE_PREFIX_PATH "/home/richa/packages/vcpkg/packages/ms-gsl_x64-linux/share/Microsoft.GSL/")
# list(APPEND CMAKE_PREFIX_PATH "/home/richa/packages/vcpkg/packages/zstd_x64-linux/")

include_directories (./src)
file (GLOB_RECURSE SourceFiles src/*.cpp)
add_executable (quail ${SourceFiles})

find_package (glog REQUIRED)
target_link_libraries (quail glog::glog)

# find_package(Microsoft.GSL CONFIG REQUIRED)
# target_link_libraries(quail Microsoft.GSL::GSL)

find_package (SEAL 3.6 REQUIRED)
target_link_libraries (quail SEAL::seal)

