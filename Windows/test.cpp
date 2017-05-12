//
// Created by steve on 5/3/17.
//

#include "SafeDriveSDK.h"

int main(int argc, char* argv[]) {
    SafeDriveSDK sdk("1.0", "variable", "en_US", Configuration::Staging, "C:\\Program Files\\SafeDrive\\");

    std::string channel = sdk.channel();
    std::string version = sdk.version();

    std::stringstream is;
    is << "SafeDriveSDK<" << channel << "> " << version;
    SafeDriveSDK::Log(is.str(), Info);

    #ifdef _WIN32
    std::string user;
    user.resize(65535);
    DWORD userSize = GetEnvironmentVariable("TEST_USER", &user[0], 65535);
    user.resize(userSize);
    #else
    std::string user = std::getenv("TEST_USER");
    #endif

    #ifdef _WIN32
    std::string password;
    password.resize(65535);
    DWORD passwordSize = GetEnvironmentVariable("TEST_PASSWORD", &password[0], 65535);
    password.resize(passwordSize);
    #else
    std::string password = std::getenv("TEST_PASSWORD");
    #endif

    std::stringstream us;

    us << "login: " << user;
    SafeDriveSDK::Log(us.str(), Info);

    std::stringstream ps;
    ps << "password: " << password;
    SafeDriveSDK::Log(ps.str(), Info);

    #ifdef _WIN32
    std::string ucid;
    ucid.resize(65535);
    DWORD ucidSize = GetEnvironmentVariable("TEST_UCID", &ucid[0], 65535);
    ucid.resize(ucidSize);
    #else
    std::string ucid = std::getenv("TEST_UCID");
    #endif

    sdk.login(user, password, ucid, [](AccountStatus status) {
        SafeDriveSDK::Log("login succeeded", Info);

        std::stringstream ss;
        ss << "account status: " << status;
        SafeDriveSDK::Log(ss.str(), Info);
    }, [](SDKException error) {
        SafeDriveSDK::Log("login failed", Info);

	    std::stringstream ss;
		SafeDriveSDK::Log(ss.str(), Info);
	});


    std::this_thread::sleep_for(std::chrono::seconds(15));
}
