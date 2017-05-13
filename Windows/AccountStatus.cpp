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
}


std::ostream & operator<<(std::ostream & os, const AccountStatus & status) {
    os << status.user_name << ":" << status.host << ":" << status.port;
    return os;
}

std::ostream& operator<<(std::ostream& os, const AccountState& state) {
    switch (state) {
        case Unknown: {
            os << "Unknown";
            break;
        }
        case Active: {
            os << "Active";
            
            break;
        }
        case Trial: {
            os << "Trial";
            
            break;
        }
        case TrialExpired: {
            os << "TrialExpired";
            
            break;
        }
        case Expired: {
            os << "Expired";
            
            break;
        }
        case Locked: {
            os << "Locked";
            
            break;
        }
        case ResetPassword: {
            os << "ResetPassword";
            
            break;
        }
        case PendingCreation: {
            os << "PendingCreation";
            
            break;
        }
    }
    return os;
}
