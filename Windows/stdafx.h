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
#include <codecvt>
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

#include <optional>

#include <atomic>
// convert UTF-8 string to wstring
static std::wstring utf8_to_wstring(const std::string& str);

// convert wstring to UTF-8 string
static std::string wstring_to_utf8(const std::wstring& str);


// convert UTF-8 string to wstring
static std::wstring utf8_to_wstring(const std::string& str) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>> myconv;
    return myconv.from_bytes(str);
}

// convert wstring to UTF-8 string
static std::string wstring_to_utf8(const std::wstring& str) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>> myconv;
    return myconv.to_bytes(str);
}
