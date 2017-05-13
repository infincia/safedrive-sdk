// stdafx.h : include file for standard system include files,
// or project specific include files that are used frequently, but
// are changed infrequently
//

#pragma once

#ifdef _WIN32
#include "targetver.h"

#define WIN32_LEAN_AND_MEAN             // Exclude rarely-used stuff from Windows headers
// Windows Header Files:
#include <windows.h>
#endif


#include <stdio.h>
#include <string>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <sstream>
#include <ostream>
#include <fstream>
#include <istream>
#include <iomanip>
#include <numeric>
#include <iterator>
#include <vector>
#include <stdexcept>
#include <thread>
#ifdef _WIN32
#include <optional>
template<typename T> using sd_optional = std::optional<T>;
template<typename T> using sd_optional_none = std::optional::nullopt

#else
#include <experimental/optional>
template<typename T> using sd_optional = std::experimental::optional<T>;
template<typename T> using sd_optional_none = std::experimental::nullopt_t;
#endif
#include <atomic>


