//
// Created by steve on 5/3/17.
//

#include "SafeDriveSDK.h"

int main(int argc, char* argv[]) {
    SafeDriveSDK sdk("1.0", "variable", "en_US", Configuration::Staging, "C:\\Program Files\\SafeDrive\\");

    std::string channel = sdk.channel();
    std::string version = sdk.version();


    std::cout << "SafeDriveSDK<" << channel << "> " << version << endl;

}
