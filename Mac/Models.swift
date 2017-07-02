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

public enum SDKSyncDirection {
    case forward
    case reverse
}

public enum SDKSyncType {
    case encrypted
    case unencrypted
}

public struct SDKSyncFolder: Equatable {
    public var id: UInt64
    public var name: String
    public var path: String
    public var date: Date
    public var encrypted: Bool
    public var type: SDKSyncType
    
    // whether the folder should be allowed to sync, is it disabled/missing etc
    public var active: Bool = true
    
    public var syncFrequency: String = "daily"
    
    public var syncTime: Date
    
    public var url: URL? {
        return URL(fileURLWithPath: path, isDirectory: true)
    }
    
    public func exists() -> Bool {
        var isDirectory: ObjCBool = false
        
        if FileManager.default.fileExists(atPath: path, isDirectory:&isDirectory) {
            if isDirectory.boolValue {
                return true
            }
        }
        return false
    }
    
    public static func == (left: SDKSyncFolder, right: SDKSyncFolder) -> Bool {
        return (left.id == right.id)
    }
    
    public init(folder: SDDKFolder) {
        self.id = folder.id
        self.name = String(cString: folder.name)
        self.path = String(cString: folder.path)
        self.date = Date(timeIntervalSince1970: Double(folder.date) / 1000)
        self.encrypted = folder.encrypted == 1 ? true : false
        self.active = folder.syncing == 1 ? true : false
        self.type = self.encrypted ? .encrypted : .unencrypted
        
        // TODO: get this from SDDKFolder once it has that property
        var components = DateComponents()
        components.hour = 0
        components.minute = 0
        let calendar = Calendar.current
        self.syncTime = calendar.date(from: components)!
    }
}

public class SDKSyncTask: Equatable {
    
    fileprivate let modificationQueue = DispatchQueue(label: "io.safedrive.SafeDriveSDK.SDKSyncTask.modificationQueue")
    
    public var folderID: UInt64 {
        get {
            var r: UInt64 = 0
            modificationQueue.sync {
                r = self._folderID
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._folderID = newValue
            })
        }
    }
    
    fileprivate var _folderID: UInt64
    
    public var name: String {
        get {
            var r: String = ""
            modificationQueue.sync {
                r = self._name
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._name = newValue
            })
        }
    }
    
    fileprivate var _name: String
    
    // start of sync
    public var syncDate: Date? {
        get {
            var r: Date?
            modificationQueue.sync {
                r = self._syncDate
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._syncDate = newValue
            })
        }
    }
    
    fileprivate var _syncDate: Date?
    
    // set to true only if sync finishes without error
    public var success: Bool {
        get {
            var r: Bool = false
            modificationQueue.sync {
                r = self._success
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._success = newValue
            })
        }
    }
    
    fileprivate var _success: Bool = false
    
    public var syncing: Bool {
        get {
            var r: Bool = false
            modificationQueue.sync {
                r = self._syncing
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._syncing = newValue
            })
        }
    }
    
    fileprivate var _syncing: Bool = false
    
    public var restoring: Bool {
        get {
            var r: Bool = false
            modificationQueue.sync {
                r = self._restoring
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._restoring = newValue
            })
        }
    }
    
    fileprivate var _restoring: Bool = false
    
    // will be NSDate() - syncDate, calculated at time of success or failure
    
    public var duration: TimeInterval {
        get {
            var r: TimeInterval = 0
            modificationQueue.sync {
                r = self._duration
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._duration = newValue
            })
        }
    }
    
    fileprivate var _duration: TimeInterval = 0
    
    // use for error messages if sync fails
    public var message: String {
        get {
            var r: String = ""
            modificationQueue.sync {
                r = self._message
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._message = newValue
            })
        }
    }
    
    fileprivate var _message: String = ""
    
    // sync progress in percentage
    public var progress: Double {
        get {
            var r: Double = 0.0
            modificationQueue.sync {
                r = self._progress
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._progress = newValue
            })
        }
    }
    
    fileprivate var _progress: Double = 0.0
    
    // sync bandwidth
    public var bandwidth: String {
        get {
            var r: String = ""
            modificationQueue.sync {
                r = self._bandwidth
            }
            return r
        }
        set (newValue) {
            modificationQueue.sync(flags: .barrier, execute: {
                self._bandwidth = newValue
            })
        }
    }
    
    fileprivate var _bandwidth: String = "0.00kB/s"
    
    required public init(folderID: UInt64, syncDate: Date, name: String) {
        _folderID = folderID
        _syncDate = syncDate
        _name = name
    }
    
    public static func == (left: SDKSyncTask, right: SDKSyncTask) -> Bool {
        return (left.folderID == right.folderID)
    }
}

public struct SDKSyncSession: Equatable {
    public var name: String
    public var size: UInt64
    public var date: Date
    public var folder_id: UInt64
    public var session_id: UInt64
        
    public init(session: SDDKSyncSession) {
        self.name = String(cString: session.name)
        self.size = session.size
        self.date = Date(timeIntervalSince1970: TimeInterval(session.date / UInt64(1000)))
        self.folder_id = session.folder_id
        self.session_id = session.session_id
    }
    
    public static func == (left: SDKSyncSession, right: SDKSyncSession) -> Bool {
        return (left.session_id == right.session_id) && (left.folder_id == right.folder_id)
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
        case SDDKAccountState_Unknown:
            self = .unknown
        case SDDKAccountState_Active:
            self = .active
        case SDDKAccountState_Trial:
            self = .trial
        case SDDKAccountState_TrialExpired:
            self = .trialExpired
        case SDDKAccountState_Locked:
            self = .locked
        case SDDKAccountState_ResetPassword:
            self = .resetPassword
        case SDDKAccountState_PendingCreation:
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
