#include "stdafx.h"
#include "SafeDriveSDK.h"
#include "sddk.h"

AccountDetails::AccountDetails(SDDKAccountDetails* cdetails) {
	assignedStorage = cdetails->assigned_storage;
	usedStorage = cdetails->used_storage;
	lowFreeStorageThreshold = cdetails->low_free_space_threshold;
	expirationDate = cdetails->expiration_date;
	cdetails = cdetails;
}

AccountDetails::~AccountDetails() {
	sddk_free_account_details(&cdetails);
}