#include "SafeDriveSDK.h"

SoftwareClient::SoftwareClient(SDDKSoftwareClient* cclient) {
	unique_client_id = cclient->unique_client_id;
	operating_system = cclient->operating_system;
	language = cclient->language;
};
