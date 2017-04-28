#include "stdafx.h"
#include "SafeDriveSDK.h"
#include "sddk.h"

AccountStatus::AccountStatus(SDDKAccountStatus* cstatus) {
	if (cstatus->status != NULL) {
		status = cstatus->status;
	}
	if (cstatus->time != NULL) {
		time = *cstatus->time;
	}
	host = cstatus->host;
	port = cstatus->port;
	user_name = cstatus->user_name;
	cstatus = cstatus;
}

AccountStatus::~AccountStatus() {
	sddk_free_account_status(&cstatus);
}

