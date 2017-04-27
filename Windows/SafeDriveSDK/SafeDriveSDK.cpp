// SafeDriveSDK.cpp : Defines the exported functions for the DLL application.
//

#include "stdafx.h"
#include "SafeDriveSDK.h"
#include "sddk.h"

SafeDriveSDK::SafeDriveSDK(std::string client_version, std::string operating_system, std::string locale, Configuration configuration, std::string storage_directory) {
	std::cout << "\n SafeDriveSDK Constructor called \n";

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
		std::cout << "Error initializing SDK: %s", error->message;
		throw SDKException();
	}
}

SafeDriveSDK::~SafeDriveSDK() {
	std::cout << "\n SafeDriveSDK Destructor called \n";
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

void SafeDriveSDK::login(std::string username, std::string password, std::string unique_client_id) {
	std::thread t1([&] {
		SDDKError* error = NULL;
		SDDKAccountStatus* status = NULL;
		int res = sddk_login(state, unique_client_id.c_str(), username.c_str(), password.c_str(), &status, &error);
		if (res == -1) {
			std::cout << "Error logging in: %s", error->message;
			sddk_free_error(&error);
			throw SDKException();
		} else {
			sddk_free_account_status(&status);
		}
	});
}

void SafeDriveSDK::load_keys(const char * phrase) {
	std::thread t1([&] {
		SDDKError * error = NULL;
		if (0 != sddk_load_keys(NULL, state, &error, phrase, NULL)) {
			std::cout << "Error loading keys: %s", error->message;
			throw SDKException();
		}
	});
}

void log(std::string message, LogLevel level) {
	std::thread t1([&] {
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
}

void SafeDriveSDK::add_folder(std::string name, std::string path) {
	std::thread t1([&] {
		SDDKError * error = NULL;
		if (0 != sddk_add_sync_folder(state, name.c_str(), path.c_str(), &error)) {
			std::cout << "Error adding folder: %s", error->message;
			throw SDKException();
		}
	});
}

void SafeDriveSDK::remove_folder(unsigned long long folderID) {
	std::thread t1([&] {
		SDDKError * error = NULL;
		if (0 != sddk_remove_sync_folder(state, folderID, &error)) {
			std::cout << "Error adding folder: %s", error->message;
			throw SDKException();
		}
	});
}

std::vector<Folder> SafeDriveSDK::get_folders() {
	SDDKFolder * folder_ptr = NULL;
	SDDKError * error = NULL;
	int64_t length = sddk_get_sync_folders(state, &folder_ptr, &error);
	if (length == -1) {
		std::cout << "Error getting folders: %s", error->message;
		throw SDKException();
	}
	SDDKFolder * head = folder_ptr;
	std::cout << "Got % folders\n";
	std::vector<Folder> folders;
	for (int i = 0; i < length; i++, folder_ptr++) {
		SDDKFolder f = *folder_ptr;
		Folder folder = Folder();
		folder.name = f.name;
		folder.path = f.path;
		folder.id = f.id;
		folder.encrypted = f.encrypted;
		folder.syncing = f.syncing;
		folders.push_back(folder);
	}
	sddk_free_folders(&head, length);
	return folders;
}
