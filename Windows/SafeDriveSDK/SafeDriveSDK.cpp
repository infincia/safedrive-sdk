// SafeDriveSDK.cpp : Defines the exported functions for the DLL application.
//

#include "stdafx.h"
#include "SafeDriveSDK.h"


// This is an example of an exported variable
SAFEDRIVESDK_API int nSafeDriveSDK=0;

// This is an example of an exported function.
SAFEDRIVESDK_API int fnSafeDriveSDK(void)
{
    return 42;
}

// This is the constructor of a class that has been exported.
// see SafeDriveSDK.h for the class definition
CSafeDriveSDK::CSafeDriveSDK()
{
    return;
}
