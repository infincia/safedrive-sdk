#include "SafeDriveSDK.h"

SDKException::SDKException(SDDKError* error) {
    type = SDKErrorType(error->error_type);
    message = error->message;
    code = error->error_type;
};
