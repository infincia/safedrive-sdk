//
//  SafeDriveSDK.cpp
//  SafeDriveSDK
//
//  Created by steve on 9/3/16.
//  Copyright © 2016 SafeDrive. All rights reserved.
//

#include "SafeDriveSDK.hpp"

#include <iostream>
#include <string>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <iterator>
#include <vector>

#include "sddk.h"


namespace SafeDriveSDK {

    class SafeDriveSDK {
        
    public:
        SafeDriveSDK(std::string storage_directory, std::string unique_client_id) {
            std::cout<<"\n SafeDriveSDK Constructor called \n";
            
            const char *s = storage_directory.c_str();
            const char *u = unique_client_id.c_str();
            state = sddk_initialize(s, u);
        }
        
        ~SafeDriveSDK() {
            std::cout<<"\n SafeDriveSDK Destructor called \n";
            sddk_free_state(&state);
        }
        
        void load_keys(unsigned char main_key[32], unsigned char hmac_key[32]) {
            sddk_load_keys(state, main_key, hmac_key);
        }
        
        bool add_folder(std::string name, std::string path) {
            int success = sddk_add_sync_folder(state, name.c_str(), path.c_str());
            switch (success) {
                case 0:
                    return true;
                default:
                    return false;
            }
        }
        
        std::vector<Folder> get_folders() {
            CFolder * folder_ptr;
            
            int64_t length = sddk_get_sync_folders(state, &folder_ptr);
            CFolder * head = folder_ptr;
            std::cout<< "Got % folders\n";
            std::vector<Folder> folders;
            for (int i = 0; i < length; i++, folder_ptr++ ) {
                CFolder f = *folder_ptr;
                Folder folder = Folder();
                folder.name = f.name;
                folder.path = f.path;
                sddk_free_string(&f.name);
                sddk_free_string(&f.path);
                folders.push_back(folder);
            }
            sddk_free_folders(&head, length);
            return folders;
        }
        
    protected:
        SDDKState * state;
    };
    
}

