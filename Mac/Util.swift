//
//  Util.swift
//  SafeDriveSDK
//
//  Created by steve on 5/1/17.
//  Copyright © 2017 SafeDrive. All rights reserved.
//

import Foundation
import SDDK
// MARK: Private

func AccountStateFromString(_ string: String) -> AccountState {
    switch string {
    case "active":
        return .active
    case "trial":
        return .trial
    case "trial-expired":
        return .trialExpired
    case "expired":
        return .expired
    case "locked":
        return .locked
    case "reset-password":
        return .resetPassword
    case "pending-creation":
        return .pendingCreation
    default:
        return .unknown
    }
}

func SDDKAccountStatusToAccountStatus(account_status: SDDKAccountStatus) -> AccountStatus {
    var s: Optional<String> = nil
    if account_status.status != nil {
        s = String(cString: account_status.status)
    }
    var t: Optional<UInt64> = nil
    if account_status.time != nil {
        t = account_status.time.pointee
    }
    let accountStatus = AccountStatus(status: s, host: String(cString: account_status.host), port: account_status.port, userName: String(cString: account_status.user_name), time: t)
    
    return accountStatus
}

func SDDKAccountDetailsToAccountDetails(account_details: SDDKAccountDetails) -> AccountDetails {
    let n: Optional<[SafeDriveNotification]> = nil
    if account_details.notifications != nil {
        let buffer = UnsafeBufferPointer<SDDKNotification>(start: UnsafePointer(account_details.notifications), count: Int(account_details.notification_count))
        let a = Array(buffer)
        var new_array = [SafeDriveNotification]()
        for notification in a {
            let title = String(cString: notification.title)
            let message = String(cString: notification.message)
            let n = SafeDriveNotification(title: title, message: message)
            new_array.append(n)
        }
        
    }
    let accountDetails = AccountDetails(assignedStorage: account_details.assigned_storage, usedStorage: account_details.used_storage, lowFreeStorageThreshold: account_details.low_free_space_threshold, expirationDate: account_details.expiration_date, notifications: n)
    
    return accountDetails
}


