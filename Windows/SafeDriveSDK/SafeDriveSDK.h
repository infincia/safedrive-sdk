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
	std::string name;
	std::string path;
	long long id;
	bool encrypted;
	bool syncing;
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


