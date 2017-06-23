//
//  Models.swift
//  SafeDriveSDK
//
//  Created by steve on 5/1/17.
//  Copyright © 2017 SafeDrive. All rights reserved.
//

import Foundation
import SDDK



public typealias SDKSuccess = () -> Void
public typealias SDKFailure = (_ error: SDKError) -> Void

public typealias SDKSyncSessionProgress = @convention(block) (_ total: UInt64, _ current: UInt64, _ new: UInt64,  _ percent: Double) -> Void

public typealias SDKSyncSessionIssue = @convention(block) (_ message: String) -> Void

public typealias SDKSaveRecoveryPhrase = @convention(block) (_ phrase: String) -> Void

public typealias SDKIssue = @convention(block) (_ message: String) -> Void


public enum SDKConfiguration {
    case Production
    case Staging
}

public class SDKSyncFolder {
    public var id: UInt64
    public var name: String
    public var path: String
    public var date: Date
    public var encrypted: Bool
    public var syncing: Bool
    
    required public init(folder: SDDKFolder) {
        self.id = folder.id
        self.name = String(cString: folder.name)
        self.path = String(cString: folder.path)
        self.date = Date(timeIntervalSince1970: Double(folder.date) / 1000)
        self.encrypted = folder.encrypted == 1 ? true : false
        self.syncing = folder.syncing == 1 ? true : false
    }
    
}

public class SDKSyncSession {
    public var name: String
    public var size: UInt64
    public var date: Date
    public var folder_id: UInt64
    public var session_id: UInt64
    
    
    required public init(session: SDDKSyncSession) {
        self.name = String(cString: session.name)
        self.size = session.size
        self.date = Date(timeIntervalSince1970: TimeInterval(session.date / UInt64(1000)))
        self.folder_id = session.folder_id
        self.session_id = session.session_id
    }
}

public class SDKAccountStatus {
    public var state: SDKAccountState
    public var host: String
    public var port: UInt16
    public var userName: String
    public var time: Optional<Date> = nil
    
    
    required public init(account_status: SDDKAccountStatus) {
        self.state = SDKAccountState(account_status.state)
        self.host = String(cString: account_status.host)
        self.port = account_status.port
        self.userName = String(cString: account_status.user_name)
        
        if account_status.time != nil {
            time = Date(timeIntervalSince1970: Double(account_status.time.pointee) / 1000)
        }        
    }
}


public enum SDKAccountState {
    case unknown         // invalid state, display error or halt
    case active          // the SFTP connection will be continued by the client
    case trial           // the SFTP connection will be continued by the client
    case trialExpired    // trial expired, trial expiration date will be returned
                         // from the server and formatted with the user's locale format
    case expired         // account expired, expiration date will be returned from
                         // the server and formatted with the user's locale format
    case locked          // account locked, date will be returned from the server
                         // and formatted with the user's locale format
    case resetPassword   // password being reset
    case pendingCreation // account not ready yet
    
    public init(_ state: SDDKAccountState) {
        switch state {
        case SDDKAccountStateUnknown:
            self = .unknown
        case SDDKAccountStateActive:
            self = .active
        case SDDKAccountStateTrial:
            self = .trial
        case SDDKAccountStateTrialExpired:
            self = .trialExpired
        case SDDKAccountStateLocked:
            self = .locked
        case SDDKAccountStateResetPassword:
            self = .resetPassword
        case SDDKAccountStatePendingCreation:
            self = .pendingCreation
        default:
            self = .unknown
        }
    }
}

extension SDKAccountState : CustomStringConvertible {
     public var description: String {
        switch self {
        case .unknown:
            return NSLocalizedString("unknown", comment: "account status")
        case .active:
            return NSLocalizedString("active", comment: "account status")
        case .trial:
            return NSLocalizedString("trial", comment: "account status")
        case .trialExpired:
            return NSLocalizedString("trial expired", comment: "account status")
        case .expired:
            return NSLocalizedString("expired", comment: "account status")
        case .locked:
            return NSLocalizedString("locked", comment: "account status")
        case .resetPassword:
            return NSLocalizedString("reset password", comment: "account status")
        case .pendingCreation:
            return NSLocalizedString("pending creation", comment: "account status")
        }
    }
}



public struct SDKSoftwareClient {
    public let uniqueClientID: String
    public let operatingSystem: String
    public let language: String
}

public class SDKAccountDetails {
    public var assignedStorage: UInt64
    public var usedStorage: UInt64
    public var lowFreeStorageThreshold: Int64
    public var expirationDate: Date
    public var notifications: Optional<[SDKSafeDriveNotification]>
    
    
    public required init(account_details: SDDKAccountDetails) {
        let n: Optional<[SDKSafeDriveNotification]> = nil
        if account_details.notifications != nil {
            let buffer = UnsafeBufferPointer<SDDKNotification>(start: UnsafePointer(account_details.notifications), count: Int(account_details.notification_count))
            let a = Array(buffer)
            var new_array = [SDKSafeDriveNotification]()
            for notification in a {
                let title = String(cString: notification.title)
                let message = String(cString: notification.message)
                let n = SDKSafeDriveNotification(title: title, message: message)
                new_array.append(n)
            }
            
        }
        
        self.assignedStorage = account_details.assigned_storage
        self.usedStorage = account_details.used_storage
        self.lowFreeStorageThreshold = account_details.low_free_space_threshold
        self.expirationDate = Date(timeIntervalSince1970: Double(account_details.expiration_date) / 1000)
        self.notifications = n
    }
}

public struct SDKSafeDriveNotification {
    public let title: String
    public let message: String
}
