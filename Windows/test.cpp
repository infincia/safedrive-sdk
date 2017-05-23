//
// Created by steve on 5/3/17.
//

#include "SafeDriveSDK.h"

int main(int argc, char* argv[]) {
#ifdef _WIN32
    SafeDriveSDK sdk(L"1.0", L"variable", L"en_US", Configuration::Staging, std::nullopt);

#else
    SafeDriveSDK sdk("1.0", "variable", "en_US", Configuration::Staging, std::experimental::nullopt);

#endif

    std::wstring channel = sdk.channel();
    std::wstring version = sdk.version();
    std::wstring storage = sdk.app_directory(Configuration::Staging);

    std::wstringstream is;
    is << L"SafeDriveSDK<" << channel << L"> " << version;
    SafeDriveSDK::Log(is.str(), Info);

#ifdef USE_KEYCHAIN
    std::wstring user = sdk.get_keychain_item(L"currentuser", L"currentuser.safedrive.io");
#else
    #ifdef _WIN32
    std::wstring user;
    user.resize(65535);
    DWORD userSize = GetEnvironmentVariable(L"TEST_USER", &user[0], 65535);
    user.resize(userSize);
    #else
    std::wstring user = std::getenv(L"TEST_USER");
    #endif
#endif

#ifdef USE_KEYCHAIN
    std::wstring password = sdk.get_keychain_item(user, L"safedrive.io");
#else
    #ifdef _WIN32
    std::wstring password;
    password.resize(65535);
    DWORD passwordSize = GetEnvironmentVariable(L"TEST_PASSWORD", &password[0], 65535);
    password.resize(passwordSize);
    #else
    std::wstring password = std::getenv(L"TEST_PASSWORD");
    #endif
#endif
    
    std::wstringstream us;

    us << L"login: " << user;
    SafeDriveSDK::Log(us.str(), Info);

    std::wstringstream ps;
    ps << L"password: " << password;
    SafeDriveSDK::Log(ps.str(), Info);

#ifdef USE_KEYCHAIN
    std::wstring ucid = sdk.get_keychain_item(user, L"ucid.safedrive.io");
#else
    #ifdef _WIN32
    std::wstring ucid;
    ucid.resize(65535);
    DWORD ucidSize = GetEnvironmentVariable(L"TEST_UCID", &ucid[0], 65535);
    ucid.resize(ucidSize);
    #else
    std::wstring ucid = std::getenv(L"TEST_UCID");
    #endif
#endif
    
    sdk.login(user, password, ucid, [](AccountStatus status) {
        SafeDriveSDK::Log(L"login succeeded", Info);

        std::wstringstream ss;
        ss << L"account status: " << status;
        SafeDriveSDK::Log(ss.str(), Info);
    }, [](SDKException error) {
        SafeDriveSDK::Log(L"login failed", Info);

        std::wstringstream ss;
        ss << L"error: " << error.message;
        SafeDriveSDK::Log(ss.str(), Error);
    });


    std::this_thread::sleep_for(std::chrono::seconds(15));
}
