#ifdef SAFEDRIVESDK_EXPORTS
#define SAFEDRIVESDK_API __declspec(dllexport)
#else
#define SAFEDRIVESDK_API __declspec(dllimport)
#endif

#include <sddk.h>


class SDKException : public std::runtime_error {
public:
	SDKException() : std::runtime_error("SDKException") { };
};

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


