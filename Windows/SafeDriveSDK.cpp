// SafeDriveSDK.cpp : Defines the exported functions for the DLL application.
//

#include "SafeDriveSDK.h"

SafeDriveSDK::SafeDriveSDK(std::string client_version, std::string operating_system, std::string locale, Configuration configuration, std::string storage_directory) {
    const char *s = storage_directory.c_str();
    const char *cv = client_version.c_str();
    const char *os = operating_system.c_str();
    const char *l = locale.c_str();

    SDDKConfiguration c;
    switch (configuration) {
    case Production:
        c = SDDKConfigurationProduction;
        break;
    case Staging:
        c = SDDKConfigurationStaging;
        break;
    }

    SDDKError * error = NULL;
    if (0 != sddk_initialize(cv, os, l, c, s, &state, &error)) {
        SDKException e(error);
        sddk_free_error(&error);
        throw e;
    }
}

bool SafeDriveSDK::ready() {
    return _ready;
}

SafeDriveSDK::~SafeDriveSDK() {
    sddk_free_state(&state);
}

std::string SafeDriveSDK::channel() {
    char* ch = sddk_get_channel();
    std::string channel(ch);
    sddk_free_string(&ch);
    return channel;
}

std::string SafeDriveSDK::version() {
    char* ver = sddk_get_version();
    std::string version(ver);
    sddk_free_string(&ver);
    return version;
}

std::string SafeDriveSDK::app_directory(Configuration configuration) {
    char* path = NULL;
    SDDKError* error = NULL;
    SDDKConfiguration c;
    switch (configuration) {
        case Production:
            c = SDDKConfigurationProduction;
            break;
        case Staging:
            c = SDDKConfigurationStaging;
            break;
    }
    
    if (0 != sddk_get_app_directory(c, &path, &error)) {
        SDKException e(error);
        throw e;
    }
    std::string s = path;
    sddk_free_string(&path);
    return s;
}



void SafeDriveSDK::login(std::string username, std::string password, std::string unique_client_id, SDKLoginSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        SDDKAccountStatus* status = NULL;
        int res = sddk_login(state, unique_client_id.c_str(), username.c_str(), password.c_str(), &status, &error);
        if (res == -1) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        } else {
            AccountStatus s = AccountStatus(status);
            sddk_free_account_status(&status);
            success(s);
        }
    });

    t1.detach();
}

void SafeDriveSDK::remove_client(std::string unique_client_id, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        if (0 != sddk_remove_client(state, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        } else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::load_keys(const char * phrase, SaveRecoveryPhrase store_phrase, Issue issue, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        void * c = static_cast<void*>(&store_phrase);
        void * c2 = static_cast<void*>(&issue);

        if (0 != sddk_load_keys(c, c2, state, &error, phrase, [](void* context, void* context2, char* new_phrase) {
            (*static_cast<SaveRecoveryPhrase*>(context))(new_phrase);
        }, [](void* context, void* context2, char* message) {
            (*static_cast<Issue*>(context))(message);

        })) {
            _ready = false;
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            _ready = true;
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_clients(std::string username, std::string password, SDKGetClientsSuccess success, SDKFailure failure) {

    std::thread t1([&] {

        SDDKSoftwareClient* clients = NULL;
        SDDKError* error = NULL;
        long long res = sddk_get_software_clients(username.c_str(), password.c_str(), &clients, &error);

        if (res == -1) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            std::vector<SoftwareClient> new_array;
            SDDKSoftwareClient* head = clients;

            for (int i = 0; i < res; i++, clients++) {
                SDDKSoftwareClient* c_client = clients;
                SoftwareClient client = SoftwareClient(c_client);
                new_array.push_back(client);
            }
            success(new_array);
            sddk_free_software_clients(&head, (unsigned long long)res);
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_account_status(SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        SDDKAccountStatus* cstatus = NULL;

        if (0 != sddk_get_account_status(state, &cstatus, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            AccountStatus s = AccountStatus(cstatus);
            sddk_free_account_status(&cstatus);
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_account_details(SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        SDDKAccountDetails* cdetails = NULL;

        if (0 != sddk_get_account_details(state, &cdetails, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            AccountDetails s = AccountDetails(cdetails);
            sddk_free_account_details(&cdetails);
            success();
        }
    });

    t1.detach();
}

std::string SafeDriveSDK::generate_unique_client_id() {
    char* unique_client_id = NULL;
    sddk_generate_unique_client_id(&unique_client_id);
    std::string s = unique_client_id;
    sddk_free_string(&unique_client_id);
    return s;
}

void SafeDriveSDK::add_folder(std::string name, std::string path, bool encrypted, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        if (0 != sddk_add_sync_folder(state, name.c_str(), path.c_str(), encrypted, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::update_folder(std::string name, std::string path, bool syncing, unsigned long long unique_id, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        unsigned char c_syncing = 0;
        if (syncing) {
            c_syncing = 1;
        }

        SDDKError * error = NULL;
        if (0 != sddk_update_sync_folder(state, name.c_str(), path.c_str(), c_syncing, unique_id, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::remove_folder(unsigned long long folderID, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        if (0 != sddk_remove_sync_folder(state, folderID, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_folder(unsigned long long folderID, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        SDDKFolder* cfolder = NULL;
        long long res = sddk_get_sync_folder(state, folderID, &cfolder, &error);
        if (res == -1) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            Folder new_folder = Folder(cfolder);
            success();
            sddk_free_folder(&cfolder);
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_folders(SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKFolder* cfolders = NULL;
        SDDKError* error = NULL;
        int64_t res = sddk_get_sync_folders(state, &cfolders, &error);
        if (res == -1) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            SDDKFolder * head = cfolders;
            std::vector<Folder> folders;
            for (int i = 0; i < res; i++, cfolders++) {
                SDDKFolder* cfolder = cfolders;
                Folder folder = Folder(cfolder);
                folders.push_back(folder);
            }
            success();
            sddk_free_folders(&head, res);
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_sessions(SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKSyncSession* csessions = NULL;
        SDDKError * error = NULL;
        int64_t res = sddk_get_sync_sessions(state, &csessions, &error);
        if (res == -1) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
            return;
        }
        else {
            SDDKSyncSession * head = csessions;
            std::vector<SyncSession> sessions;
            for (int i = 0; i < res; i++, csessions++) {
                SDDKSyncSession* csession = csessions;
                SyncSession session = SyncSession(csession);
                sessions.push_back(session);
            }
            success();
            sddk_free_sync_sessions(&head, res);
        }
    });

    t1.detach();
}

void SafeDriveSDK::remove_session(unsigned long long session_id, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        if (0 != sddk_remove_sync_session(state, session_id, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::cancel_sync_task(std::string session_name, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        if (0 != sddk_cancel_sync_task(session_name.c_str(), &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::sync_folder(unsigned long long folder_id, std::string session_name, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        void * c_progress = static_cast<void*>(&progress);
        void * c_issue = static_cast<void*>(&issue);

        unsigned int res = sddk_sync(c_progress,
            c_issue,
            state,
            &error,
            session_name.c_str(),
            folder_id,
            [](void* context, void* context2, unsigned long long total, unsigned long long current, unsigned long long new_bytes, double percent, unsigned int  tick) {
                (*static_cast<SyncSessionProgress*>(context))(total, current, new_bytes, percent);
            },
            [](void* context, void* context2, char const* message) {
                (*static_cast<SyncSessionIssue*>(context2))(message);
            });

        if (0 != res) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::restore_folder(unsigned long long folder_id, std::string session_name, std::string destination, unsigned long long session_size, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        void * c_progress = static_cast<void*>(&progress);
        void * c_issue = static_cast<void*>(&issue);

        int res = sddk_restore(c_progress,
            c_issue,
            state,
            &error,
            session_name.c_str(),
            folder_id,
            destination.c_str(),
            session_size,
            [](void* context, void* context2, unsigned long long total, unsigned long long current, unsigned long long new_bytes, double percent, unsigned int  tick) {
                (*static_cast<SyncSessionProgress*>(context))(total, current, new_bytes, percent);
            },
            [](void* context, void* context2, char const* message) {
                (*static_cast<SyncSessionIssue*>(context2))(message);
            });

        if (0 != res) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}

void SafeDriveSDK::Log(std::string message, LogLevel level) {
    std::thread t1([message, level] {
        const char *m = message.c_str();

        unsigned char l = 0;
        switch (level) {
        case Error:
            l = 0;
            break;
        case Warn:
            l = 1;
            break;
        case Info:
            l = 2;
            break;
        case Debug:
            l = 3;
            break;
        case Trace:
            l = 4;
            break;
        }

        sddk_log(m, l);
    });

    t1.detach();
}


void SafeDriveSDK::report_error(std::exception exc, std::string context, std::string description, std::string unique_client_id, sd_optional<std::string> operating_system, sd_optional<std::string> client_version, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        const char* c_client_version = NULL;
        if (client_version) {
            c_client_version = (*client_version).c_str();
        }
        const char* c_operating_system = NULL;
        if (operating_system) {
            c_operating_system = (*operating_system).c_str();
        }
        if (0 != sddk_report_error(c_client_version, 
                                   c_operating_system, 
                                   unique_client_id.c_str(), 
                                   description.c_str(), 
                                   context.c_str(),
                                   &error)) {
            std::cout << "Error reporting error: " << error->message << std::endl;
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            success();
        }
    });

    t1.detach();
}
