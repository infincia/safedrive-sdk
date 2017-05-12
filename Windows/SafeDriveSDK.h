#ifdef _WIN32
#ifdef SAFEDRIVESDK_EXPORTS
#define SAFEDRIVESDK_API
#else
#define SAFEDRIVESDK_API
#endif
#else
#define SAFEDRIVESDK_API
#endif

#include "stdafx.h"
#include <sddk.h>

enum SAFEDRIVESDK_API Configuration {
	Production,
	Staging,
};

enum SAFEDRIVESDK_API LogLevel {
	Error,
	Warn,
	Info,
	Debug,
	Trace,
};

class SAFEDRIVESDK_API Folder {
public:
	long long id;
	std::string name;
	std::string path;
	unsigned long long date;
	bool encrypted;
	bool syncing;
	Folder(SDDKFolder* cfolder);
};

class SAFEDRIVESDK_API SyncSession {
public:
	std::string name;
	unsigned long long size;
	long long date;
	unsigned long long folder_id;
	unsigned long long session_id;
	SyncSession(SDDKSyncSession* csyncsession);
};

enum SAFEDRIVESDK_API AccountState {
    Unknown,
    Active,
    Trial,
    TrialExpired,
    Expired,
    Locked,
    ResetPassword,
    PendingCreation,
};

class SAFEDRIVESDK_API AccountStatus {
public:
    AccountState state;
	std::string host;
	unsigned short port;
	std::string user_name;
    sd_optional<long long> time;
	AccountStatus(SDDKAccountStatus* cstatus);
	~AccountStatus();
	friend ostream& operator<<(ostream& os, const AccountStatus& status);

private:
	//SDDKAccountStatus* cstatus;
};

class SAFEDRIVESDK_API SoftwareClient {
public:
	std::string unique_client_id;
	std::string operating_system;
	std::string language;
	SoftwareClient(SDDKSoftwareClient* cclient);
};

class SAFEDRIVESDK_API AccountDetails {
public:
	unsigned long long assignedStorage;
	unsigned long long usedStorage;
	long long lowFreeStorageThreshold;
	unsigned long long expirationDate;
	AccountDetails(SDDKAccountDetails* cdetails);
	~AccountDetails();
private:
	SDDKAccountDetails* cdetails;
};

class SAFEDRIVESDK_API SafeDriveNotification {
public:
	std::string title;
	std::string message;
};

enum SAFEDRIVESDK_API SDKErrorType {
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

class SAFEDRIVESDK_API SDKException : public std::runtime_error {
public:
	SDKErrorType type();
	std::string message();
	int code();
	SDKException(SDDKError* error);
	~SDKException();
private:
	SDDKError* error;
};

typedef std::function<void()> SDKSuccess;
typedef std::function<void(AccountStatus status)> SDKLoginSuccess;
typedef std::function<void(std::vector<SoftwareClient>)> SDKGetClientsSuccess;
typedef std::function<void(SDKException error)> SDKFailure;
typedef std::function<void(unsigned long long total, unsigned long long current, unsigned long long new_bytes, double percent)> SyncSessionProgress;
typedef std::function<void(std::string message)> SyncSessionIssue;
typedef std::function<void(std::string message)> SaveRecoveryPhrase;
typedef std::function<void(std::string message)> Issue;



class SAFEDRIVESDK_API SafeDriveSDK {
public:
	SafeDriveSDK(std::string client_version, std::string operating_system, std::string locale, Configuration configuration, std::string storage_directory);
	~SafeDriveSDK();
	std::string channel();
	std::string version();
	void login(std::string username, std::string password, std::string unique_client_id, std::function<void(SDDKAccountStatus status)> success, SDKFailure failure);
	void get_clients(std::string username, std::string password, std::function<void(std::vector<SoftwareClient>)> success, SDKFailure failure);
	void remove_client(std::string unique_client_id, SDKSuccess success, SDKFailure failure);
	void get_account_status(SDKSuccess success, SDKFailure failure);
	void get_account_details(SDKSuccess success, SDKFailure failure);
	std::string generate_unique_client_id();
	void load_keys(const char * phrase, SaveRecoveryPhrase store_phrase, Issue issue, SDKSuccess success, SDKFailure failure);
	static void Log(std::string message, LogLevel level);
	void add_folder(std::string name, std::string path, bool encrypted, SDKSuccess success, SDKFailure failure);
	void update_folder(std::string name, std::string path, bool syncing, unsigned long long unique_id, SDKSuccess success, SDKFailure failure);
	void remove_folder(unsigned long long folderID, SDKSuccess success, SDKFailure failure);
	void get_folder(unsigned long long folderID, SDKSuccess success, SDKFailure failure);
	void get_folders(SDKSuccess success, SDKFailure failure);
	void get_sessions(SDKSuccess success, SDKFailure failure);
	void remove_session(unsigned long long session_id, SDKSuccess success, SDKFailure failure);
	void cancel_sync_task(std::string session_name, SDKSuccess success, SDKFailure failure);
	void sync_folder(unsigned long long folder_id, std::string session_name, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure);
	void restore_folder(unsigned long long folder_id, std::string session_name, std::string destination, unsigned long long session_size, SyncSessionProgress progress, SyncSessionIssue issue, SDKSuccess success, SDKFailure failure);
	void report_error(std::exception exc, std::string context, std::string description, std::string unique_client_id, sd_optional<std::string> operating_system, sd_optional<std::string> client_version, SDKSuccess success, SDKFailure failure);
	bool ready();
private:
	SDDKState * state;
	std::atomic<bool> _ready = ATOMIC_VAR_INIT(false);
};


