cmake_minimum_required(VERSION 3.15)

add_subdirectory(JUCE)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Debug" CACHE STRING "Choose the type of build, options are: Debug, Release, or MinSizeRel." FORCE)
    message(STATUS "CMAKE_BUILD_TYPE not set, defaulting to Debug.")
endif()

project(JUCE_SYS VERSION 0.0.1)

add_library(JuceRustBindings STATIC)

target_sources(JuceRustBindings
        PRIVATE
        src/bindings.cpp)

target_include_directories(JuceRustBindings PUBLIC $ENV{CPAL_ASIO_DIR}\\common)

target_compile_definitions(JuceRustBindings
        PUBLIC
        # JUCE_WEB_BROWSER and JUCE_USE_CURL would be on by default, but you might not need them.
        JUCE_DISPLAY_SPLASH_SCREEN=0
        JUCE_WEB_BROWSER=0  # If you remove this, add `NEEDS_WEB_BROWSER TRUE` to the `juce_add_plugin` call
        JUCE_USE_CURL=0     # If you remove this, add `NEEDS_CURL TRUE` to the `juce_add_plugin` call
        JUCE_VST3_CAN_REPLACE_VST2=0
        JUCE_USE_SSE_INTRINSICS=0
        JUCE_STANDALONE_APPLICATION=1
        JUCE_ASIO=1)

target_link_libraries(JuceRustBindings
        PRIVATE
        juce::juce_audio_devices
        PUBLIC
        juce::juce_recommended_config_flags
        juce::juce_recommended_lto_flags
        juce::juce_recommended_warning_flags)

set_target_properties(JuceRustBindings PROPERTIES
        POSITION_INDEPENDENT_CODE TRUE
        VISIBILITY_INLINES_HIDDEN TRUE
        C_VISIBILITY_PRESET hidden
        CXX_VISIBILITY_PRESET hidden)
