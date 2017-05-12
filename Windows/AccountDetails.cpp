#include "stdafx.h"
#include "SafeDriveSDK.h"
#include "sddk.h"


AccountDetails::AccountDetails(SDDKAccountDetails* details) {
	assignedStorage = details->assigned_storage;
	usedStorage = details->used_storage;
	lowFreeStorageThreshold = details->low_free_space_threshold;
	expirationDate = details->expiration_date;
	cdetails = details;
}

AccountDetails::~AccountDetails() {
	//sddk_free_account_details(&cdetails);
}
