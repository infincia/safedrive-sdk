//
// Created by steve on 5/3/17.
//

#include "SafeDriveSDK.h"

int main(int argc, char* argv[]) {
    SafeDriveSDK sdk("1.0", "variable", "en_US", Configuration::Staging, std::experimental::nullopt);

    std::string channel = sdk.channel();
    std::string version = sdk.version();
    std::string storage = sdk.app_directory(Configuration::Staging);

    std::stringstream is;
    is << "SafeDriveSDK<" << channel << "> " << version;
    SafeDriveSDK::Log(is.str(), Info);

#ifdef USE_KEYCHAIN
    std::string user = sdk.get_keychain_item("currentuser", "currentuser.safedrive.io");
#else
    #ifdef _WIN32
    std::string user;
    user.resize(65535);
    DWORD userSize = GetEnvironmentVariable("TEST_USER", &user[0], 65535);
    user.resize(userSize);
    #else
    std::string user = std::getenv("TEST_USER");
    #endif
#endif

#ifdef USE_KEYCHAIN
    std::string password = sdk.get_keychain_item(user, "safedrive.io");

#else
    #ifdef _WIN32
    std::string password;
    password.resize(65535);
    DWORD passwordSize = GetEnvironmentVariable("TEST_PASSWORD", &password[0], 65535);
    password.resize(passwordSize);
    #else
    std::string password = std::getenv("TEST_PASSWORD");
    #endif
#endif
    
    std::stringstream us;

    us << "login: " << user;
    SafeDriveSDK::Log(us.str(), Info);

    std::stringstream ps;
    ps << "password: " << password;
    SafeDriveSDK::Log(ps.str(), Info);

#ifdef USE_KEYCHAIN
    std::string ucid = sdk.get_keychain_item(user, "ucid.safedrive.io");
#else
    #ifdef _WIN32
    std::string ucid;
    ucid.resize(65535);
    DWORD ucidSize = GetEnvironmentVariable("TEST_UCID", &ucid[0], 65535);
    ucid.resize(ucidSize);
    #else
    std::string ucid = std::getenv("TEST_UCID");
    #endif
#endif
    
    sdk.login(user, password, ucid, [](AccountStatus status) {
        SafeDriveSDK::Log("login succeeded", Info);

        std::stringstream ss;
        ss << "account status: " << status;
        SafeDriveSDK::Log(ss.str(), Info);
    }, [](SDKException error) {
        SafeDriveSDK::Log("login failed", Info);

        std::stringstream ss;
        ss << "error: " << error.message;
        SafeDriveSDK::Log(ss.str(), Error);
    });


    std::this_thread::sleep_for(std::chrono::seconds(15));
}
