#include "SafeDriveSDK.h"

AccountStatus::AccountStatus(SDDKAccountStatus* status) {
    switch (status->state) {
        case SDDKAccountStateUnknown: {
            state = Unknown;
            break;
        }
        case SDDKAccountStateActive: {
            state = Active;
            break;
        }
        case SDDKAccountStateTrial: {
            state = Trial;
            break;
        }
        case SDDKAccountStateTrialExpired: {
            state = TrialExpired;
            break;
        }
        case SDDKAccountStateExpired: {
            state = Expired;
            break;
        }
        case  SDDKAccountStateLocked: {
            state = Locked;
            break;
        }
        case SDDKAccountStateResetPassword: {
            state = ResetPassword;
            break;
        }
        case SDDKAccountStatePendingCreation: {
            state = PendingCreation;
            break;
        }
    }

    if (cstatus->time != NULL) {
		time = *status->time;
	}
	host = cstatus->host;
	port = cstatus->port;
	user_name = cstatus->user_name;
	cstatus = status;
}

AccountStatus::~AccountStatus() {
	sddk_free_account_status(&cstatus);
}

