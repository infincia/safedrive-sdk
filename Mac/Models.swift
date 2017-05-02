//
//  Models.swift
//  SafeDriveSDK
//
//  Created by steve on 5/1/17.
//  Copyright © 2017 SafeDrive. All rights reserved.
//

import Foundation



public typealias SDKSuccess = () -> Void
public typealias SDKFailure = (_ error: SDKError) -> Void

public typealias SyncSessionProgress = @convention(block) (_ total: UInt64, _ current: UInt64, _ new: UInt64,  _ percent: Double) -> Void

public typealias SyncSessionIssue = @convention(block) (_ message: String) -> Void

public typealias SaveRecoveryPhrase = @convention(block) (_ phrase: String) -> Void

public typealias Issue = @convention(block) (_ message: String) -> Void


public enum SDKConfiguration {
    case Production
    case Staging
}

public struct Folder {
    public let id: UInt64
    public let name: String
    public let path: String
    public let date: UInt64
    public let encrypted: Bool
    public let syncing: Bool
    
}

public struct SDSyncSession {
    public let name: String
    public let size: UInt64
    public let date: Date
    public let folder_id: UInt64
    public let session_id: UInt64
}

public struct AccountStatus {
    public let state: AccountState
    public let host: String
    public let port: UInt16
    public let userName: String
    public let time: Optional<UInt64>
}


public enum AccountState {
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
}

extension AccountState : CustomStringConvertible {
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



public struct SoftwareClient {
    public let uniqueClientID: String
    public let operatingSystem: String
    public let language: String
}

public struct AccountDetails {
    public let assignedStorage: UInt64
    public let usedStorage: UInt64
    public let lowFreeStorageThreshold: Int64
    public let expirationDate: UInt64
    public let notifications: Optional<[SafeDriveNotification]>
}

public struct SafeDriveNotification {
    public let title: String
    public let message: String
}
