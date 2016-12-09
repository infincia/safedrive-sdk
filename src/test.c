#import "stdio.h"
#import <stdlib.h>
#import <string.h>
#import "sdsync.h"

static CContext *context = NULL;

int add(const char * name, const char * path) {
	return sdsync_add_sync_folder(context, name, path);
}

int main ( int argc, char **argv ) {
    char * unique_client_id = malloc((64 * sizeof(char)) + 1);

    int ret = sdsync_get_unique_client_id("stephen@safedrive.io", &unique_client_id);
    int l = strlen(unique_client_id);
    printf("C<test/main>: got unique id: ");

    for (int i = 0; i < l; ++i) {
        printf("%c", unique_client_id[i]);
    }
    printf("\n");

    context = sdsync_initialize("/Users/steve/Library/Application Support/SafeDrive", unique_client_id);

    free(unique_client_id);

    uint8_t * main_key = "ab03c34f1ece08211fe2a8039fd6424199b3f5d7b55ff13b1134b364776c45c5";
    uint8_t * hmac_key = "63d6ff853569a0aadec5f247bba51786bb73494d1a06bdc036ebac5034a2920b";
    sdsync_load_keys(context, main_key, hmac_key);

    sdsync_load_credentials(context, "username", "password", "sftp-client.safedrive.io", "22");

    int result = add("Documents", "/Users/steve/Documents");
    if (result != 0) {
        printf("C<test/main>: Failed to add folder: %d\n", result);
    }

    CFolder * folder_ptr;
    int64_t length = sdsync_get_sync_folders(context, &folder_ptr);
    CFolder * head = folder_ptr;
    printf("C<test/main>: found %lld folders\n", length);
    for (int i = 0; i < length; i++, folder_ptr++) {
        CFolder folder = *folder_ptr;
        printf("C<test/main>: folder <%s, %s>\n", folder.name, folder.path);
        sdsync_free_string(&folder.name);
        sdsync_free_string(&folder.path);
        sdsync_create_archive(context, folder.id);
    }
    sdsync_free_folders(&head, length);
    sdsync_free_context(&context);
    return 0;
}
