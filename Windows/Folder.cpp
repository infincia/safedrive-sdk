#include "SafeDriveSDK.h"
#include "sddk.h"

Folder::Folder(SDDKFolder* cfolder) {
	id = cfolder->id;
	name = cfolder->name;
	path = cfolder->path;
	date = cfolder->date;
	encrypted = cfolder->encrypted == 1 ? true : false;
	syncing = cfolder->syncing == 1 ? true : false;
}
