#pragma once
#include "stdafx.h"
#include <sddk.h>

enum Configuration {
    Production,
    Staging,
};

enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
};

class SyncFolder {
public:
    long long id;
    std::wstring name;
    std::wstring path;
    unsigned long long date;
    bool encrypted;
    bool syncing;
    SyncFolder(SDDKFolder* cfolder);
    SyncFolder();
};

class SyncSession {
public:
    std::wstring name;
    unsigned long long size;
    long long date;
    unsigned long long folder_id;
    unsigned long long session_id;
    SyncSession(SDDKSyncSession* csyncsession);
};

enum AccountState {
    Unknown,
    Active,
    Trial,
    TrialExpired,
    Expired,
    Locked,
    ResetPassword,
    PendingCreation,
};

class AccountStatus {
public:
    AccountState state;
    std::wstring host;
    unsigned short port;
    std::wstring user_name;
    std::optional<long long> time;
    AccountStatus(SDDKAccountStatus* cstatus);
    friend std::ostream& operator<<(std::ostream& os, const AccountStatus& status);
};

class SoftwareClient {
public:
    std::wstring unique_client_id;
    std::wstring operating_system;
    std::wstring language;
    SoftwareClient(SDDKSoftwareClient* cclient);
};


class AccountDetails {
public:
    unsigned long long assignedStorage;
    unsigned long long usedStorage;
    long long lowFreeStorageThreshold;
    unsigned long long expirationDate;
    AccountDetails(SDDKAccountDetails* cdetails);
};

enum SDKErrorType {
    SDKErrorTypeStateMissing = 0x0000,
    SDKErrorTypeInternal = 0x0001,
    SDKErrorTypeRequestFailure = 0x0002,
    SDKErrorTypeNetworkFailure = 0x0003,
    SDKErrorTypeConflict = 0x0004,
    SDKErrorTypeBlockMissing = 0x0005,
    SDKErrorTypeSessionMissing = 0x0006,
    SDKErrorTypeRecoveryPhraseIncorrect = 0x0007,
    SDKErrorTypeInsufficientFreeSpace = 0x0008,
    SDKErrorTypeAuthentication = 0x0009,
    SDKErrorTypeUnicodeError = 0x000A,
    SDKErrorTypeTokenExpired = 0x000B,
    SDKErrorTypeCryptoError = 0x000C,
    SDKErrorTypeIO = 0x000D,
    SDKErrorTypeSyncAlreadyInProgress = 0x000E,
    SDKErrorTypeRestoreAlreadyInProgress = 0x000F,
    SDKErrorTypeExceededRetries = 0x0010,
    SDKErrorTypeKeychainError = 0x0011,
    SDKErrorTypeBlockUnreadable = 0x0012,
    SDKErrorTypeSessionUnreadable = 0x0013,
    SDKErrorTypeServiceUnavailable = 0x0014,
    SDKErrorTypeCancelled = 0x0015,
    SDKErrorTypeFolderMissing = 0x0016,
    SDKErrorTypeKeyCorrupted = 0x0017,
};

class SDKException {
public:
    SDKException(SDDKError* error);
    SDKErrorType type;
    std::wstring message;
    int code;
};

typedef std::function<void()> SDKSuccess;
typedef std::function<void(AccountStatus status)> SDKAccountStatusSuccess;
typedef std::function<void(AccountDetails details)> SDKAccountDetailsSuccess;
typedef std::function<void(std::vector<SoftwareClient>)> SDKGetClientsSuccess;
typedef std::function<void(SDKException error)> SDKFailure;
typedef std::function<void(unsigned long long total, unsigned long long current, unsigned long long new_bytes, double percent)> SyncSessionProgress;
typedef std::function<void(std::wstring message)> SyncSessionIssue;
typedef std::function<void(std::wstring message)> SaveRecoveryPhrase;
typedef std::function<void(std::wstring message)> Issue;

class SafeDriveSDK {
public:
    SafeDriveSDK(std::wstring client_version, std::wstring operating_system, std::wstring locale, Configuration configuration, std::optional<std::wstring> storage_directory);
    ~SafeDriveSDK();
    std::wstring channel();
    std::wstring version();
    void login(std::wstring username, std::wstring password, std::wstring unique_client_id, SDKAccountStatusSuccess success, SDKFailure failure);
    void get_clients(std::wstring username, std::wstring password, SDKGetClientsSuccess success, SDKFailure failure);
    void remove_client(std::wstring unique_client_id, SDKSuccess success, SDKFailure failure);
    std::wstring app_directory(Configuration configuration);
    void get_account_status(SDKAccountStatusSuccess success, SDKFailure failure);
    void get_account_details(SDKAccountDetailsSuccess success, SDKFailure failure);
    std::wstring generate_unique_client_id();
    void load_keys(const char * phrase, SaveRecoveryPhrase store_phrase, Issue issue, SDKSuccess success, SDKFailure failure);
    static void Log(std::wstring message, LogLevel level);
    void add_folder(std::wstring name, std::wstring path, bool encrypted, SDKSuccess success, SDKFailure failure);
    void update_folder(std::wstring name, std::wstring path, bool syncing, unsigned long long unique_id, SDKSuccess success, SDKFailure failure);
    void remove_folder(unsigned long long folderID, SDKSuccess success, SDKFailure failure);
    void get_folder(unsigned long long folderID, SDKSuccess success, SDKFailure failure);
    void get_folders(SDKSuccess success, SDKFailure failure);
    void get_sessions(SDKSuccess success, SDKFailure failure);
    void remove_session(unsigned long long session_id, SDKSuccess success, SDKFailure failure);
    void cancel_sync_task(std::wstring session_name, SDKSuccess success, SDKFailure failure);
    void sync_folder(unsigned long long folder_id, std::wstring session_name, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure);
    void restore_folder(unsigned long long folder_id, std::wstring session_name, std::wstring destination, unsigned long long session_size, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure);
    void report_error(std::exception exc, std::wstring context, std::wstring description, std::wstring unique_client_id, std::optional<std::wstring> operating_system, std::optional<std::wstring> client_version, SDKSuccess success, SDKFailure failure);
    bool ready();
    std::wstring get_keychain_item(std::wstring username, std::wstring service);
    void delete_keychain_item(std::wstring username, std::wstring service);
    void set_keychain_item(std::wstring username, std::wstring service, std::wstring secret);


private:
    SDDKState * state;
    std::atomic<bool> _ready = ATOMIC_VAR_INIT(false);
};


