#ifdef SAFEDRIVESDK_EXPORTS
#define SAFEDRIVESDK_API __declspec(dllexport)
#else
#define SAFEDRIVESDK_API __declspec(dllimport)
#endif

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
	std::string path;
	unsigned long long date;
	bool encrypted;
	bool syncing;
};

class SAFEDRIVESDK_API SyncSession {
	std::string name;
	unsigned long long size;
	long long date;
	unsigned long long folder_id;
	unsigned long long session_id;
};

class SAFEDRIVESDK_API AccountStatus {
	std::optional<std::string> status;
	std::string host;
	unsigned short port;
	std::string user_name;
	std::optional<long long> time;
};

class SAFEDRIVESDK_API SoftwareClient {
	std::string unique_client_id;
	std::string operating_system;
	std::string language;
};

class SAFEDRIVESDK_API AccountDetails {
	unsigned long long assignedStorage;
	unsigned long long usedStorage;
	long long lowFreeStorageThreshold;
	unsigned long long expirationDate;
	std::optional<std::vector<SafeDriveNotification>> notifications;
};

class SAFEDRIVESDK_API SafeDriveNotification {
	std::string title;
	std::string message;
};

enum SAFEDRIVESDK_API SDKErrorType {
	StateMissing = 0x0000,
	Internal = 0x0001,
	RequestFailure = 0x0002,
	NetworkFailure = 0x0003,
	Conflict = 0x0004,
	BlockMissing = 0x0005,
	SessionMissing = 0x0006,
	RecoveryPhraseIncorrect = 0x0007,
	InsufficientFreeSpace = 0x0008,
	Authentication = 0x0009,
	UnicodeError = 0x000A,
	TokenExpired = 0x000B,
	CryptoError = 0x000C,
	IO = 0x000D,
	SyncAlreadyInProgress = 0x000E,
	RestoreAlreadyInProgress = 0x000F,
	ExceededRetries = 0x0010,
	KeychainError = 0x0011,
	BlockUnreadable = 0x0012,
	SessionUnreadable = 0x0013,
	ServiceUnavailable = 0x0014,
	Cancelled = 0x0015,
	FolderMissing = 0x0016,
	KeyCorrupted = 0x0017,
};

class SAFEDRIVESDK_API SDKException : public std::runtime_error {
public:
	std::string message;
	SDKErrorType kind;
	int code() {
		return this->kind;
	};
	SDKException(SDKErrorType kind, std::string message);
	SDKException(SDDKError sdkError);
};

class SAFEDRIVESDK_API SafeDriveSDK {
public:
	SafeDriveSDK(std::string client_version, std::string operating_system, std::string locale, Configuration configuration, std::string storage_directory);
	~SafeDriveSDK();
	std::string SafeDriveSDK::channel();
	std::string SafeDriveSDK::version();
	void login(std::string username, std::string password, std::string unique_client_id);
	void load_keys(const char * phrase);
	void log(std::string message, LogLevel level);
	void add_folder(std::string name, std::string path);
	void remove_folder(unsigned long long folderID);
	std::vector<Folder> get_folders();
protected:
	SDDKState * state;
};


