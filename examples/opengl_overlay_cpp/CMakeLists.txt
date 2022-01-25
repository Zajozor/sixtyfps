# Copyright © SixtyFPS GmbH <info@sixtyfps.io>
# SPDX-License-Identifier: (GPL-3.0-only OR LicenseRef-SixtyFPS-commercial)

cmake_minimum_required(VERSION 3.14)
project(opengl_overlay_cpp LANGUAGES CXX)

if (NOT TARGET SixtyFPS::SixtyFPS)
    find_package(SixtyFPS REQUIRED)
endif()

add_executable(opengl_overlay main.cpp)
target_link_libraries(opengl_overlay PRIVATE SixtyFPS::SixtyFPS OpenGL::GL OpenGL::EGL)
sixtyfps_target_60_sources(opengl_overlay scene.60)
