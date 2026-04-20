cmake_minimum_required(VERSION 3.16)
project(hello_gateorix_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(hello_gateorix_cpp main.cpp)

# Link Winsock on Windows
if(WIN32)
    target_link_libraries(hello_gateorix_cpp PRIVATE ws2_32)
endif()

# Install to a known location for the Tauri sidecar to find
install(TARGETS hello_gateorix_cpp DESTINATION bin)
