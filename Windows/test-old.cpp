// test.cpp : Defines the entry point for the console application.
//

#include "stdafx.h"

#include "sddk.h"

static SDDKState *state = NULL;

void progress(double percent) {
	printf("C<test/progress>: %f\n", percent);
}

void store_recovery_key_cb(void* context, char* new_phrase) {
	printf("C<test/store_recovery_key_cb>: %s", new_phrase);
}

int main() {

	char * username = NULL;
	SDDKError * current_user_error = NULL;
	if (0 != sddk_get_keychain_item("currentuser", "currentuser.safedrive.io", &username, &current_user_error)) {
		printf("C<test/main>: Failed to get current user: %s", current_user_error->message);
		sddk_free_error(&current_user_error);
		return 1;
	}
	printf("C<test/main>: user: %s", username);

	char * ucid = NULL;
	SDDKError * ucid_error = NULL;
	if (0 != sddk_get_keychain_item(username, "ucid.safedrive.io", &ucid, &ucid_error)) {
		printf("C<test/main>: Failed to get current ucid: %s", ucid_error->message);
		sddk_free_error(&ucid_error);
		return 1;
	}
	printf("C<test/main>: ucid: %s", ucid);


	char * password = NULL;
	SDDKError * password_error = NULL;
	if (0 != sddk_get_keychain_item(username, "safedrive.io", &password, &password_error)) {
		printf("C<test/main>: Failed to get password: %s", password_error->message);
		sddk_free_error(&password_error);
		return 1;
	}
	printf("C<test/main>: pass: %s", password);

	SDDKError * init_error = NULL;
	if (0 != sddk_initialize("1.0", "Windows", "EN_us", SDDKConfigurationStaging, "C:\\Users\\steve", &state, &init_error)) {
		printf("C<test/main>: Failed to initialize sddk: %s", init_error->message);
		sddk_free_error(&init_error);
		return 1;
	}

	SDDKAccountStatus *status = NULL;
	SDDKError * login_error = NULL;
	if (0 != sddk_login(state, ucid, username, password, &status, &login_error)) {
		printf("C<test/main>: Failed to login: %s", login_error->message);
		sddk_free_error(&login_error);
		return 1;
	}

	SDDKError * keys_error = NULL;
	if (0 != sddk_load_keys(NULL, state, &keys_error, NULL, store_recovery_key_cb)) {
		printf("C<test/main>: Failed to load keys: %s", keys_error->message);
		sddk_free_error(&keys_error);
		return 1;
	}

	SDDKError * add_folder_error = NULL;
	if (0 != sddk_add_sync_folder(state, "Documents", "C:\\Users\\steve\\My Documents", &add_folder_error)) {
		printf("C<test/main>: Failed to add folder: %s", add_folder_error->message);
		sddk_free_error(&add_folder_error);
		return 1;
	}

	SDDKError * get_folders_error = NULL;
	SDDKFolder * folder_ptr;
	int64_t length = sddk_get_sync_folders(state, &folder_ptr, &get_folders_error);
	if (length == -1) {
		printf("C<test/main>: Failed to get folder: %s", get_folders_error->message);
		sddk_free_error(&get_folders_error);
		return 1;
	}

	SDDKFolder * head = folder_ptr;
	printf("C<test/main>: found %lld folders\n", length);
	for (int i = 0; i < length; i++, folder_ptr++) {
		SDDKFolder folder = *folder_ptr;
		printf("C<test/main>: folder <%s, %s>\n", folder.name, folder.path);
		//if (0 != sddk_create_archive(state, folder.name, folder.path, folder.id, &progress)) {
		//    printf("C<test/main>: Failed to sync folder\n");
		//}
	}

	sddk_free_string(&username);
	sddk_free_string(&password);
	sddk_free_string(&ucid);
	sddk_free_account_status(&status);

	sddk_free_folders(&head, length);
	sddk_free_state(&state);
	return 0;
}

