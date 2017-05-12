#include "SafeDriveSDK.h"
#include "sddk.h"

SyncSession::SyncSession(SDDKSyncSession* csyncsession) {
	name = csyncsession->name;
	size = csyncsession->size;
	date = csyncsession->date;
	folder_id = csyncsession->folder_id;
	session_id = csyncsession->session_id;
}
