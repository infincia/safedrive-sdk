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
    
    if (status->time) {
        time = *status->time;
    }
    host = status->host;
    port = status->port;
    user_name = status->user_name;
    //cstatus = status;
}

AccountStatus::~AccountStatus() {
    //sddk_free_account_status(&cstatus);
}


ostream & operator<<(ostream & os, const AccountStatus & status) {
    os << status.user_name << ":" << status.host << ":" << status.port;
    return os;
}

