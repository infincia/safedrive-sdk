//
//  SafeDriveSDK.swift
//  SafeDriveSDK
//
//  Created by steve on 8/23/16.
//  Copyright © 2016 SafeDrive. All rights reserved.
//

import Foundation

import sddk

public enum SDKConfiguration {
    case Production
    case Staging
}

public struct Folder {
    public let id: UInt32
    public let name: String
    public let path: String
}

public struct SyncSession {
    public let name: String
	public let size: UInt64;
	public let date: Date;
	public let folder_id: UInt32;
}

public struct AccountStatus {
    public let status: Optional<String>
    public let host: String
    public let port: UInt16
    public let userName: String
    public let time: Optional<UInt64>
}

public struct AccountDetails {
    public let assignedStorage: UInt64
    public let usedStorage: UInt64
    public let lowFreeStorageThreshold: Int64
    public let expirationDate: UInt64
    public let notifications: Optional<[Notification]>
}

public struct Notification {
    public let title: String
    public let message: String
}

public enum SDKErrorType {
    case StateMissing
    case Internal
    case RequestFailure
    case NetworkFailure
    case Conflict
    case BlockMissing
    case SessionMissing
    case RecoveryPhraseIncorrect
    case InsufficientFreeSpace
    case Authentication
    case UnicodeError
    case TokenExpired
    case CryptoError
    case IO
    case SyncAlreadyInProgress
    case RestoreAlreadyInProgress
    case ExceededRetries
}

public struct SDKError: Error {
    public var message: String
    public var kind: SDKErrorType
}



func SDKErrorFromSDDKError(sdkError: SDDKError) -> SDKError {
    let s = String(cString: sdkError.message!)
    var type: SDKErrorType
    
    switch sdkError.error_type {
    case Internal:
        type = SDKErrorType.Internal
    case RequestFailure:
        type = SDKErrorType.RequestFailure
    case NetworkFailure:
        type = SDKErrorType.NetworkFailure
    case Conflict:
        type = SDKErrorType.Conflict
    case BlockMissing:
        type = SDKErrorType.BlockMissing
    case SessionMissing:
        type = SDKErrorType.SessionMissing
    case RecoveryPhraseIncorrect:
        type = SDKErrorType.RecoveryPhraseIncorrect
    case InsufficientFreeSpace:
        type = SDKErrorType.InsufficientFreeSpace
    case Authentication:
        type = SDKErrorType.Authentication
    case UnicodeError:
        type = SDKErrorType.UnicodeError
    case TokenExpired:
        type = SDKErrorType.TokenExpired
    case CryptoError:
        type = SDKErrorType.CryptoError
    case IO:
        type = SDKErrorType.IO
    case SyncAlreadyInProgress:
        type = SDKErrorType.SyncAlreadyInProgress
    case RestoreAlreadyInProgress:
        type = SDKErrorType.RestoreAlreadyInProgress
    case ExceededRetries:
        type = SDKErrorType.ExceededRetries
    default:
        exit(1)
        break
    }

    return SDKError(message: s, kind: type)
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
    let n: Optional<[Notification]> = nil
    if account_details.notifications != nil {
        let buffer = UnsafeBufferPointer<SDDKNotification>(start: UnsafePointer(account_details.notifications), count: Int(account_details.notification_count))
        let a = Array(buffer)
        var new_array = [Notification]()
        for notification in a {
            let title = String(cString: notification.title)
            let message = String(cString: notification.message)
            let n = Notification(title: title, message: message)
            new_array.append(n)
        }
        
    }
    let accountDetails = AccountDetails(assignedStorage: account_details.assigned_storage, usedStorage: account_details.used_storage, lowFreeStorageThreshold: account_details.low_free_space_threshold, expirationDate: account_details.expiration_date, notifications: n)

    return accountDetails
}

public typealias SyncSessionSuccess = @convention(block) () -> Void
public typealias SyncSessionProgress = @convention(block) (_ total: UInt32, _ current: UInt32, _ progress: Double) -> Void
public typealias SyncSessionFailure = @convention(block) (_ error: Error) -> Void

public typealias SaveRecoveryPhrase = @convention(block) (_ phrase: String) -> Void

public class SafeDriveSDK: NSObject {

    public static let sharedSDK = SafeDriveSDK()

    var state: OpaquePointer? = nil

    var folders = [Folder]()
    
    var sessions = [SyncSession]()

    public override init() {
        super.init()
    }

    public func setUp(client_version: String, operating_system: String, config: SDKConfiguration) throws {
        var sddk_config: SDDKConfiguration
        switch config {
        case .Production:
            sddk_config = SDDKConfigurationProduction
        case .Staging:
            sddk_config = SDDKConfigurationStaging
        }
        self.state = sddk_initialize(client_version, operating_system, sddk_config)
    }
    
    deinit {
        sddk_free_state(&state)
    }
    
    public func login(_ username: String, password: String, local_storage_path: String, unique_client_id: String) throws -> AccountStatus {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil
        var status: UnsafeMutablePointer<SDDKAccountStatus>? = nil

        let res = sddk_login(state, local_storage_path, unique_client_id, username, password, &status, &error)
        defer {
            if res == -1 {
                sddk_free_error(&error)
            } else {
                sddk_free_account_status(&status)
            }
        }
        switch res {
        case 0:
            return SDDKAccountStatusToAccountStatus(account_status: status!.pointee)
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }

    public func loadKeys(_ recoveryPhrase: String?, storePhrase: @escaping SaveRecoveryPhrase) throws {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil

        let res = sddk_load_keys(unsafeBitCast(storePhrase, to: UnsafeMutableRawPointer.self), state, &error, recoveryPhrase) { (context, phrase) in
            // call back to Swift to save the phrase somewhere
            let b = unsafeBitCast(context, to: SaveRecoveryPhrase.self)
            let p = String(cString: phrase!)
            var m = phrase
            b(p)
            sddk_free_string(&m)
        }
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            return
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }
    
    public func getAccountStatus() throws -> AccountStatus {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var account_status_ptr: UnsafeMutablePointer<SDDKAccountStatus>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_get_account_status(state, &account_status_ptr, &error)
        defer {
            if res >= 0 {
                sddk_free_account_status(&account_status_ptr)
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            return SDDKAccountStatusToAccountStatus(account_status: account_status_ptr!.pointee)
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }
    
    public func getAccountDetails() throws -> AccountDetails {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var account_details_ptr: UnsafeMutablePointer<SDDKAccountDetails>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_get_account_details(state, &account_details_ptr, &error)
        defer {
            if res >= 0 {
                sddk_free_account_details(&account_details_ptr)
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            return SDDKAccountDetailsToAccountDetails(account_details: account_details_ptr!.pointee)
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }
    
    public func uniqueClientID(_ email_address: String) throws -> Optional<String> {
        var unique_client_id: UnsafeMutablePointer<CChar>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil

        let res = sddk_get_unique_client_id(email_address, &unique_client_id, &error)
        defer {
            if res >= 0 {
                sddk_free_string(&unique_client_id)
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            return String(cString: unique_client_id!)
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }

    public func addFolder(_ name: String, path: String) throws {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil

        let res = sddk_add_sync_folder(state, name, path, &error)
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            return
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }
    
    public func removeFolder(_ folderId: UInt32) throws {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_remove_sync_folder(state, folderId, &error)
        
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        
        switch res {
        case 0:
            return
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }
    
    public func getFolder(folderId: UInt32) throws -> Folder {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        var folder_ptr: UnsafeMutablePointer<SDDKFolder>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_get_sync_folder(state, folderId, &folder_ptr, &error)
        defer {
            if res >= 0 {
                sddk_free_folders(&folder_ptr, UInt64(res))
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            let name = String(cString: folder_ptr!.pointee.name)
            let path = String(cString: folder_ptr!.pointee.path)
            let id = folder_ptr!.pointee.id
            let folder = Folder(id: id, name: name, path: path)
        
            return folder
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }

    public func getFolders() throws -> [Folder] {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        
        var folder_ptr: UnsafeMutablePointer<SDDKFolder>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_get_sync_folders(state, &folder_ptr, &error)
        defer {
            if res >= 0 {
                sddk_free_folders(&folder_ptr, UInt64(res))
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            let buffer = UnsafeBufferPointer<SDDKFolder>(start: UnsafePointer(folder_ptr), count: Int(res))
            let a = Array(buffer)
            var new_array = [Folder]()
            for folder in a {
                let name = String(cString: folder.name)
                let path = String(cString: folder.path)
                let id = folder.id
                let new_folder = Folder(id: id, name: name, path: path)
                new_array.append(new_folder)
            }
            
            self.folders = new_array
            return folders
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }

    }
    
    public func getSessions() throws -> [SyncSession] {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        
        var sessions_ptr: UnsafeMutablePointer<SDDKSyncSession>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_get_sync_sessions(state, &sessions_ptr, &error)
        defer {
            if res >= 0 {
                sddk_free_sync_sessions(&sessions_ptr, UInt64(res))
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            let buffer = UnsafeBufferPointer<SDDKSyncSession>(start: UnsafePointer(sessions_ptr), count: Int(res))
            let a = Array(buffer)
            var new_array = [SyncSession]()
            for session in a {
                let name = String(cString: session.name)
                let size = session.size
                let ti = (session.date / UInt64(1000))
                let date: Date = Date(timeIntervalSince1970: TimeInterval(ti))
                let id = session.folder_id
                let new_session = SyncSession(name: name, size: size, date: date, folder_id: id)
                new_array.append(new_session)
            }
            self.sessions = new_array
            return sessions
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
        

    }
    
    public func syncFolder(folderID: UInt32, sessionName: String, progress: @escaping SyncSessionProgress, success: @escaping SyncSessionSuccess, failure: @escaping SyncSessionFailure) {
        guard let state = self.state else {
            let e = SDKError(message: "State missing, cannot continue", kind: .StateMissing)
            failure(e)
            return
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        DispatchQueue.global(priority: .default).async {
            let res = sddk_sync(unsafeBitCast(progress, to: UnsafeMutableRawPointer.self), state, &error, sessionName, folderID, { (context, total, current, percent, tick) in
                // call back to Swift to report progress
            
                let b = unsafeBitCast(context, to: SyncSessionProgress.self)
                b(total, current, percent)
            })
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                success()
            default:
                failure(SDKErrorFromSDDKError(sdkError: error!.pointee))
            }
        }

    }
    
    public func restoreFolder(folderID: UInt32, sessionName: String, destination: URL, progress: @escaping SyncSessionProgress, success: @escaping SyncSessionSuccess, failure: @escaping SyncSessionFailure) {
        guard let state = self.state else {
            let e = SDKError(message: "State missing, cannot continue", kind: .StateMissing)
            failure(e)
            return
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        DispatchQueue.global(priority: .default).async {
            let res = sddk_restore(unsafeBitCast(progress, to: UnsafeMutableRawPointer.self), state, &error, sessionName, folderID, destination.path, { (context, total, current, percent, tick) in
                // call back to Swift to report progress
            
                let b = unsafeBitCast(context, to: SyncSessionProgress.self)
                b(total, current, percent)
            })
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                success()
            default:
                failure(SDKErrorFromSDDKError(sdkError: error!.pointee))
            }
        }

    }
    
    
    
    
    // gc
    
    public func gc() throws {
        guard let state = self.state else {
            throw SDKError(message: "State missing, cannot continue", kind: .StateMissing)
        }
        
        var error: UnsafeMutablePointer<SDDKError>? = nil

        let res = sddk_gc(state, &error)
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            return
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
    }
    
}

