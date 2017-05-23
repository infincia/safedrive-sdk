// SafeDriveSDK.cpp : Defines the exported functions for the DLL application.
//

#include "SafeDriveSDK.h"

SafeDriveSDK::SafeDriveSDK(std::wstring client_version, std::wstring operating_system, std::wstring locale, Configuration configuration, std::optional<std::wstring> storage_directory) {
    const char *s;
    
    if (storage_directory) {
        s = wstring_to_utf8((*storage_directory)).c_str();
    } else {
        s = NULL;
    }
    const char *cv = wstring_to_utf8(client_version).c_str();
    const char *os = wstring_to_utf8(operating_system).c_str();
    const char *l = wstring_to_utf8(locale).c_str();

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

std::wstring SafeDriveSDK::channel() {
    char* ch = sddk_get_channel();
    std::wstring channel(utf8_to_wstring(ch));
    sddk_free_string(&ch);
    return channel;
}

std::wstring SafeDriveSDK::version() {
    char* ver = sddk_get_version();
    std::wstring version(utf8_to_wstring(ver));
    sddk_free_string(&ver);
    return version;
}

std::wstring SafeDriveSDK::app_directory(Configuration configuration) {
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
    std::wstring s = utf8_to_wstring(path);
    sddk_free_string(&path);
    return s;
}



void SafeDriveSDK::login(std::wstring username, std::wstring password, std::wstring unique_client_id, SDKAccountStatusSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        SDDKAccountStatus* status = NULL;
        int res = sddk_login(state, wstring_to_utf8(unique_client_id).c_str(), wstring_to_utf8(username).c_str(), wstring_to_utf8(password).c_str(), &status, &error);
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

void SafeDriveSDK::remove_client(std::wstring unique_client_id, SDKSuccess success, SDKFailure failure) {
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
            (*static_cast<SaveRecoveryPhrase*>(context))(utf8_to_wstring(new_phrase));
        }, [](void* context, void* context2, char* message) {
            (*static_cast<Issue*>(context))(utf8_to_wstring(message));

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

void SafeDriveSDK::get_clients(std::wstring username, std::wstring password, SDKGetClientsSuccess success, SDKFailure failure) {

    std::thread t1([&] {

        SDDKSoftwareClient* clients = NULL;
        SDDKError* error = NULL;
        long long res = sddk_get_software_clients(wstring_to_utf8(username).c_str(), wstring_to_utf8(password).c_str(), &clients, &error);

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

void SafeDriveSDK::get_account_status(SDKAccountStatusSuccess success, SDKFailure failure) {
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
            success(s);
        }
    });

    t1.detach();
}

void SafeDriveSDK::get_account_details(SDKAccountDetailsSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError* error = NULL;
        SDDKAccountDetails* cdetails = NULL;

        if (0 != sddk_get_account_details(state, &cdetails, &error)) {
            SDKException e(error);
            sddk_free_error(&error);
            failure(e);
        }
        else {
            AccountDetails d = AccountDetails(cdetails);
            sddk_free_account_details(&cdetails);
            success(d);
        }
    });

    t1.detach();
}

std::wstring SafeDriveSDK::generate_unique_client_id() {
    char* unique_client_id = NULL;
    sddk_generate_unique_client_id(&unique_client_id);
    std::wstring s = utf8_to_wstring(unique_client_id);
    sddk_free_string(&unique_client_id);
    return s;
}

void SafeDriveSDK::add_folder(std::wstring name, std::wstring path, bool encrypted, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        if (0 != sddk_add_sync_folder(state, wstring_to_utf8(name).c_str(), wstring_to_utf8(path).c_str(), encrypted, &error)) {
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

void SafeDriveSDK::update_folder(std::wstring name, std::wstring path, bool syncing, unsigned long long unique_id, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        unsigned char c_syncing = 0;
        if (syncing) {
            c_syncing = 1;
        }

        SDDKError * error = NULL;
        if (0 != sddk_update_sync_folder(state, wstring_to_utf8(name).c_str(), wstring_to_utf8(path).c_str(), c_syncing, unique_id, &error)) {
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
            SyncFolder new_folder = SyncFolder(cfolder);
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
            std::vector<SyncFolder> folders;
            for (int i = 0; i < res; i++, cfolders++) {
                SDDKFolder* cfolder = cfolders;
                SyncFolder folder = SyncFolder(cfolder);
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

void SafeDriveSDK::cancel_sync_task(std::wstring session_name, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        if (0 != sddk_cancel_sync_task(wstring_to_utf8(session_name).c_str(), &error)) {
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

void SafeDriveSDK::sync_folder(unsigned long long folder_id, std::wstring session_name, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        void * c_progress = static_cast<void*>(&progress);
        void * c_issue = static_cast<void*>(&issue);

        unsigned int res = sddk_sync(c_progress,
            c_issue,
            state,
            &error,
            wstring_to_utf8(session_name).c_str(),
            folder_id,
            [](void* context, void* context2, unsigned long long total, unsigned long long current, unsigned long long new_bytes, double percent, unsigned int  tick) {
                (*static_cast<SyncSessionProgress*>(context))(total, current, new_bytes, percent);
            },
            [](void* context, void* context2, char const* message) {
                (*static_cast<SyncSessionIssue*>(context2))(utf8_to_wstring(message));
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

void SafeDriveSDK::restore_folder(unsigned long long folder_id, std::wstring session_name, std::wstring destination, unsigned long long session_size, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        void * c_progress = static_cast<void*>(&progress);
        void * c_issue = static_cast<void*>(&issue);

        int res = sddk_restore(c_progress,
            c_issue,
            state,
            &error,
            wstring_to_utf8(session_name).c_str(),
            folder_id,
            wstring_to_utf8(destination).c_str(),
            session_size,
            [](void* context, void* context2, unsigned long long total, unsigned long long current, unsigned long long new_bytes, double percent, unsigned int  tick) {
                (*static_cast<SyncSessionProgress*>(context))(total, current, new_bytes, percent);
            },
            [](void* context, void* context2, char const* message) {
                (*static_cast<SyncSessionIssue*>(context2))(utf8_to_wstring(message));
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

void SafeDriveSDK::Log(std::wstring message, LogLevel level) {
    std::thread t1([message, level] {
        const char *m = wstring_to_utf8(message).c_str();

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


void SafeDriveSDK::report_error(std::exception exc, std::wstring context, std::wstring description, std::wstring unique_client_id, std::optional<std::wstring> operating_system, std::optional<std::wstring> client_version, SDKSuccess success, SDKFailure failure) {
    std::thread t1([&] {
        SDDKError * error = NULL;
        const char* c_client_version = NULL;
        if (client_version) {
            c_client_version = wstring_to_utf8((*client_version)).c_str();
        }
        const char* c_operating_system = NULL;
        if (operating_system) {
            c_operating_system = wstring_to_utf8((*operating_system)).c_str();
        }
        if (0 != sddk_report_error(c_client_version, 
                                   c_operating_system, 
                                   wstring_to_utf8(unique_client_id).c_str(),
                                   wstring_to_utf8(description).c_str(),
                                   wstring_to_utf8(context).c_str(),
                                   &error)) {
            std::wcout << L"Error reporting error: " << utf8_to_wstring(error->message) << std::endl;
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


std::wstring SafeDriveSDK::get_keychain_item(std::wstring username, std::wstring service) {
    char* secret = NULL;
    SDDKError* error = NULL;
    
    if (0 != sddk_get_keychain_item(wstring_to_utf8(username).c_str(), wstring_to_utf8(service).c_str(), &secret, &error)) {
        SDKException e(error);
        sddk_free_error(&error);
        throw e;
    }

    std::wstring s = utf8_to_wstring(secret);
    sddk_free_string(&secret);
    return s;
}

void SafeDriveSDK::delete_keychain_item(std::wstring username, std::wstring service) {
    SDDKError* error = NULL;
    if (0 != sddk_delete_keychain_item(wstring_to_utf8(username).c_str(), wstring_to_utf8(service).c_str(), &error)) {
        SDKException e(error);
        sddk_free_error(&error);
        throw e;
    }
}

void SafeDriveSDK::set_keychain_item(std::wstring username, std::wstring service, std::wstring secret) {
    SDDKError* error = NULL;
    if (0 != sddk_set_keychain_item(wstring_to_utf8(username).c_str(), wstring_to_utf8(service).c_str(), wstring_to_utf8(secret).c_str(), &error)) {
        SDKException e(error);
        sddk_free_error(&error);
        throw e;
    }
}


AccountDetails::AccountDetails(SDDKAccountDetails* details) {
    assignedStorage = details->assigned_storage;
    usedStorage = details->used_storage;
    lowFreeStorageThreshold = details->low_free_space_threshold;
    expirationDate = details->expiration_date;
}

AccountStatus::AccountStatus(SDDKAccountStatus* status) {
    switch (status->state) {
        case SDDKAccountStateUnknown: {
            state = Unknown;
            break;
        }
        case SDDKAccountStateActive: {
            state = Active;
            break;
        }
        case SDDKAccountStateTrial: {
            state = Trial;
            break;
        }
        case SDDKAccountStateTrialExpired: {
            state = TrialExpired;
            break;
        }
        case SDDKAccountStateExpired: {
            state = Expired;
            break;
        }
        case  SDDKAccountStateLocked: {
            state = Locked;
            break;
        }
        case SDDKAccountStateResetPassword: {
            state = ResetPassword;
            break;
        }
        case SDDKAccountStatePendingCreation: {
            state = PendingCreation;
            break;
        }
    }

    if (status->time) {
        time = *status->time;
    }
    host = utf8_to_wstring(status->host);
    port = status->port;
    user_name = utf8_to_wstring(status->user_name);
}


std::wostream & operator<<(std::wostream & os, const AccountStatus & status) {
    os << status.user_name << L":" << status.host << L":" << status.port;
    return os;
}

std::wostream& operator<<(std::wostream& os, const AccountState& state) {
    switch (state) {
        case Unknown: {
            os << L"Unknown";
            break;
        }
        case Active: {
            os << L"Active";

            break;
        }
        case Trial: {
            os << L"Trial";

            break;
        }
        case TrialExpired: {
            os << L"TrialExpired";

            break;
        }
        case Expired: {
            os << L"Expired";

            break;
        }
        case Locked: {
            os << L"Locked";

            break;
        }
        case ResetPassword: {
            os << L"ResetPassword";

            break;
        }
        case PendingCreation: {
            os << L"PendingCreation";

            break;
        }
    }
    return os;
}

SDKException::SDKException(SDDKError* error) {
    type = SDKErrorType(error->error_type);
    message = utf8_to_wstring(error->message);
    code = error->error_type;
};

SoftwareClient::SoftwareClient(SDDKSoftwareClient* cclient) {
    unique_client_id = utf8_to_wstring(cclient->unique_client_id);
    operating_system = utf8_to_wstring(cclient->operating_system);
    language = utf8_to_wstring(cclient->language);
};

SyncFolder::SyncFolder() {
    id = (long long)1;
    name = L"";
    path = L"D:\\";
    date = (unsigned long long)0;
    encrypted = true;
    syncing = false;
}


SyncFolder::SyncFolder(SDDKFolder* cfolder) {
    id = cfolder->id;
    name = utf8_to_wstring(cfolder->name);
    path = utf8_to_wstring(cfolder->path);
    date = cfolder->date;
    encrypted = cfolder->encrypted == 1 ? true : false;
    syncing = cfolder->syncing == 1 ? true : false;
}

SyncSession::SyncSession(SDDKSyncSession* csyncsession) {
    name = utf8_to_wstring(csyncsession->name);
    size = csyncsession->size;
    date = csyncsession->date;
    folder_id = csyncsession->folder_id;
    session_id = csyncsession->session_id;
}



