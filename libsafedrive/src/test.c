#import "stdio.h"
#import <stdlib.h>
#import <string.h>
#import "sddk.h"

static SDDKContext *context = NULL;


int add(const char * name, const char * path) {
	return sddk_add_sync_folder(context, name, path);
}

void store_recovery_key_cb(char const* new_phrase) {
    printf("C<test/store_recovery_key_cb>: ");
    int l = strlen(new_phrase);

    for (int i = 0; i < l; ++i) {
        printf("%c", new_phrase[i]);
    }
    printf("\n");

}

void progress(double percent) {
    printf("C<test/progress>: %f\n", percent);
}

int main ( int argc, char **argv ) {
    char * unique_client_id = malloc((64 * sizeof(char)) + 1);

    char username[] = "user";
    char password[] = "password";

    char recovery_phrase[] = "phrase";

    int ret = sddk_get_unique_client_id(username, &unique_client_id);
    int l = strlen(unique_client_id);
    printf("C<test/main>: got unique id: ");

    for (int i = 0; i < l; ++i) {
        printf("%c", unique_client_id[i]);
    }
    printf("\n");

    context = sddk_initialize("/Users/steve/Library/Application Support/SafeDrive", unique_client_id);

    free(unique_client_id);

    if (0 != sddk_login(context, username, password)) {
        printf("C<test/main>: Failed to login\n");
        return 1;
    }

    sddk_load_keys(context, recovery_phrase, &store_recovery_key_cb);

    int result = add("Documents", "/Users/steve/Documents");
    if (result != 0) {
        printf("C<test/main>: Failed to add folder: %d\n", result);
    }

    SDDKFolder * folder_ptr;
    int64_t length = sddk_get_sync_folders(context, &folder_ptr);
    SDDKFolder * head = folder_ptr;
    printf("C<test/main>: found %lld folders\n", length);
    for (int i = 0; i < length; i++, folder_ptr++) {
        SDDKFolder folder = *folder_ptr;
        printf("C<test/main>: folder <%s, %s>\n", folder.name, folder.path);
        //if (0 != sddk_create_archive(context, folder.name, folder.path, folder.id, &progress)) {
        //    printf("C<test/main>: Failed to sync folder\n");
        //}
    }
    sddk_free_folders(&head, length);
    sddk_free_context(&context);
    return 0;
}
