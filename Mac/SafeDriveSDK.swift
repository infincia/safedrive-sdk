//
//  SafeDriveSDK.swift
//  SafeDriveSDK
//
//  Created by steve on 8/23/16.
//  Copyright © 2016 SafeDrive. All rights reserved.
//

import Foundation

import SDDK

public class SafeDriveSDK: NSObject {

    public static let sharedSDK = SafeDriveSDK()
    
    fileprivate let readyQueue = DispatchQueue(label: "io.safedrive.readyQueue")
    
    fileprivate var _ready = false


    public var ready: Bool {
        get {
            var r: Bool?
            readyQueue.sync {
                r = self._ready
            }
            return r!
        }
        set (newValue) {
            readyQueue.sync(flags: .barrier, execute: {
                self._ready = newValue
            })
        }
    }
    
    public static var sddk_channel: String {
        var ch = sddk_get_channel()
        defer {
            sddk_free_string(&ch)
        }
        return String(cString: ch!)
    }
    
    public static var sddk_version: String {
        var ver = sddk_get_version()
        defer {
            sddk_free_string(&ver)
        }
        return String(cString: ver!)
    }

    var state: OpaquePointer? = nil

    public override init() {
        super.init()
    }

    public func setUp(client_version: String, operating_system: String, language_code: String, config: SDKConfiguration, local_storage_path: String) throws {
        var sddk_config: SDDKConfiguration
        switch config {
        case .Production:
            sddk_config = SDDKConfigurationProduction
        case .Staging:
            sddk_config = SDDKConfigurationStaging
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil
        var state: OpaquePointer? = nil
            
        let res = sddk_initialize(client_version, operating_system, language_code, sddk_config, local_storage_path, &state, &error)
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case 0:
            self.state = state
        default:
            throw SDKErrorFromSDDKError(sdkError: error!.pointee)
        }
        
    }
    
    deinit {
        sddk_free_state(&state)
    }
    
    public func login(_ username: String, password: String, unique_client_id: String, completionQueue queue: DispatchQueue, success: @escaping (_ status: AccountStatus) -> Void, failure: @escaping SDKFailure) {

        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil
            var status: UnsafeMutablePointer<SDDKAccountStatus>? = nil
            
            let res = sddk_login(self.state, unique_client_id, username, password, &status, &error)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                } else {
                    sddk_free_account_status(&status)
                }
            }
            switch res {
            case 0:
                let s = SDDKAccountStatusToAccountStatus(account_status: status!.pointee)
                queue.async { success(s) }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func removeClient(completionQueue queue: DispatchQueue, success: @escaping () -> Void, failure: @escaping SDKFailure) {

        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil
            var status: UnsafeMutablePointer<SDDKAccountStatus>? = nil
            
            let res = sddk_remove_client(self.state, &error)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }

    public func loadKeys(_ recoveryPhrase: String?, completionQueue queue: DispatchQueue, storePhrase: @escaping SaveRecoveryPhrase, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
    
        DispatchQueue.global(priority: .default).async {
            
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_load_keys(unsafeBitCast(storePhrase, to: UnsafeMutableRawPointer.self), self.state!, &error, recoveryPhrase) { (context, phrase) in
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
                self.ready = true
                queue.async { success() }
            default:
                self.ready = false
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func getClients(withUser username: String, password: String, completionQueue queue: DispatchQueue, success: @escaping (_ folders: [SoftwareClient]) -> Void, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {

            var clients_ptr: UnsafeMutablePointer<SDDKSoftwareClient>? = nil
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_get_software_clients(username, password, &clients_ptr, &error)
            defer {
                if res >= 0 {
                    sddk_free_software_clients(&clients_ptr, UInt64(res))
                }
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case -1:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            default:
                let buffer = UnsafeBufferPointer<SDDKSoftwareClient>(start: UnsafePointer(clients_ptr), count: Int(res))
                let a = Array(buffer)
                var new_array = [SoftwareClient]()
                for c_client in a {
                    let uniqueClientId = String(cString: c_client.unique_client_id)
                    let language = String(cString: c_client.language)
                    let operatingSystem = String(cString: c_client.operating_system)
                    let client = SoftwareClient(uniqueClientID: uniqueClientId, operatingSystem: operatingSystem, language: language)
                    new_array.append(client)
                }
                
                queue.async { success(new_array) }

            }
        }

    }
    
    public func getAccountStatus(completionQueue queue: DispatchQueue, success: @escaping (_ status: AccountStatus) -> Void, failure: @escaping SDKFailure) {

        DispatchQueue.global(priority: .default).async {
        
            var account_status_ptr: UnsafeMutablePointer<SDDKAccountStatus>? = nil
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_get_account_status(self.state!, &account_status_ptr, &error)
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
                let s = SDDKAccountStatusToAccountStatus(account_status: account_status_ptr!.pointee)
                queue.async { success(s) }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func getAccountDetails(completionQueue queue: DispatchQueue, success: @escaping (_ details: AccountDetails) -> Void, failure: @escaping SDKFailure) {

        DispatchQueue.global(priority: .default).async {
        
            var account_details_ptr: UnsafeMutablePointer<SDDKAccountDetails>? = nil
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_get_account_details(self.state!, &account_details_ptr, &error)
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
                let d = SDDKAccountDetailsToAccountDetails(account_details: account_details_ptr!.pointee)
                queue.async { success(d) }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func generateUniqueClientID() -> String {
        var unique_client_id: UnsafeMutablePointer<CChar>? = nil

        sddk_generate_unique_client_id(&unique_client_id)
        defer {
            sddk_free_string(&unique_client_id)
        }
        
        return String(cString: unique_client_id!)
    }
    
    public func addFolder(_ name: String, path: String, encrypted: Bool, completionQueue queue: DispatchQueue, success: @escaping (_ folderId: UInt64) -> Void, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            var c_encrypted: UInt8 = 0
            if encrypted {
                c_encrypted = 1
            }
            
            let res = sddk_add_sync_folder(self.state!, name, path, c_encrypted, &error)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case -1:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            default:
                queue.async { success(UInt64(res)) }
            }
        }
    }
    
    public func updateFolder(_ name: String, path: String, syncing: Bool, uniqueID: UInt64, completionQueue queue: DispatchQueue, success: @escaping () -> Void, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            var c_syncing: UInt8 = 0
            if syncing {
                c_syncing = 1
            }
            
            let res = sddk_update_sync_folder(self.state!, name, path, c_syncing, uniqueID, &error)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case -1:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            default:
                queue.async { success() }
            }
        }
    }
    
    public func removeFolder(_ folderId: UInt64, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        DispatchQueue.global(priority: .default).async {
            
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_remove_sync_folder(self.state!, folderId, &error)
            
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func getFolder(folderId: UInt64, completionQueue queue: DispatchQueue, success: @escaping (_ folder: Folder) -> Void, failure: @escaping SDKFailure)  {

        DispatchQueue.global(priority: .default).async {
            
            var folder_ptr: UnsafeMutablePointer<SDDKFolder>? = nil
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_get_sync_folder(self.state!, folderId, &folder_ptr, &error)
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
                let new_folder = Folder(id: id, name: name, path: path, date: folder_ptr!.pointee.date, encrypted: folder_ptr!.pointee.encrypted == 1 ? true : false, syncing: folder_ptr!.pointee.syncing == 1 ? true : false)
                
                queue.async { success(new_folder) }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }

    public func getFolders(completionQueue queue: DispatchQueue, success: @escaping (_ folders: [Folder]) -> Void, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {

            var folder_ptr: UnsafeMutablePointer<SDDKFolder>? = nil
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_get_sync_folders(self.state!, &folder_ptr, &error)
            defer {
                if res >= 0 {
                    sddk_free_folders(&folder_ptr, UInt64(res))
                }
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case -1:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            default:
                let buffer = UnsafeBufferPointer<SDDKFolder>(start: UnsafePointer(folder_ptr), count: Int(res))
                let a = Array(buffer)
                var new_array = [Folder]()
                for folder in a {
                    let name = String(cString: folder.name)
                    let path = String(cString: folder.path)
                    let id = folder.id
                    let new_folder = Folder(id: id, name: name, path: path, date: folder.date, encrypted: folder.encrypted == 1 ? true : false, syncing: folder.syncing == 1 ? true : false)
                    new_array.append(new_folder)
                }
                
                queue.async { success(new_array) }

            }
        }

    }
    
    public func getSessions(completionQueue queue: DispatchQueue, success: @escaping (_ sessions: [SDSyncSession]) -> Void, failure: @escaping SDKFailure) {
    
        DispatchQueue.global(priority: .default).async {

            var sessions_ptr: UnsafeMutablePointer<SDDKSyncSession>? = nil
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_get_sync_sessions(self.state!, &sessions_ptr, &error)
            defer {
                if res >= 0 {
                    sddk_free_sync_sessions(&sessions_ptr, UInt64(res))
                }
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case -1:
               let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            default:
                let buffer = UnsafeBufferPointer<SDDKSyncSession>(start: UnsafePointer(sessions_ptr), count: Int(res))
                let a = Array(buffer)
                var new_array = [SDSyncSession]()
                for session in a {
                    let name = String(cString: session.name)
                    let size = session.size
                    let ti = (session.date / UInt64(1000))
                    let date: Date = Date(timeIntervalSince1970: TimeInterval(ti))
                    let id = session.folder_id
                    let session_id = session.session_id
                    let new_session = SDSyncSession(name: name, size: size, date: date, folder_id: id, session_id: session_id)
                    new_array.append(new_session)
                }

                queue.async { success(new_array) }
            }
        }

    }
    
    public func removeSession(_ sessionId: UInt64, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        DispatchQueue.global(priority: .default).async {
            
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_remove_sync_session(self.state!, sessionId, &error)
            
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func cancelSyncTask(sessionName: String, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil

            let res = sddk_cancel_sync_task(sessionName, &error)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func syncFolder(folderID: UInt64, sessionName: String, completionQueue queue: DispatchQueue, progress: @escaping SyncSessionProgress, issue: @escaping SyncSessionIssue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil

            let res = sddk_sync(unsafeBitCast(progress, to: UnsafeMutableRawPointer.self),
                                unsafeBitCast(issue, to: UnsafeMutableRawPointer.self),
                                self.state!,
                                &error,
                                sessionName,
                                folderID,
                                { (context, context2, total, current, new, percent, tick) in
                // call back to Swift to report progress
                let b = unsafeBitCast(context, to: SyncSessionProgress.self)
                b(total, current, new, percent)
            }, { (context, context2, message) in
                let m = String(cString: message!)

                let b = unsafeBitCast(context2, to: SyncSessionIssue.self)
                b(m)
            })
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }

    }
    
    public func restoreFolder(folderID: UInt64, sessionName: String, destination: URL, sessionSize: UInt64, completionQueue queue: DispatchQueue, progress: @escaping SyncSessionProgress, issue: @escaping SyncSessionIssue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
        
            var error: UnsafeMutablePointer<SDDKError>? = nil

            let res = sddk_restore(unsafeBitCast(progress, to: UnsafeMutableRawPointer.self),
                                   unsafeBitCast(issue, to: UnsafeMutableRawPointer.self),
                                   self.state!,
                                   &error,
                                   sessionName,
                                   folderID,
                                   destination.path,
                                   sessionSize,
                                   { (context, context2, total, current, new, percent, tick) in
                // call back to Swift to report progress
                let b = unsafeBitCast(context, to: SyncSessionProgress.self)
                b(total, current, new, percent)
            }, { (context, context2, message) in
                let m = String(cString: message!)

                let b = unsafeBitCast(context2, to: SyncSessionIssue.self)
                b(m)
            })
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }

    }
    
    public func log(_ message: String, _ level: SDKLogLevel) {
        DispatchQueue.global(priority: .default).async {
            sddk_log(message, level.rawValue)
        }
    }
    
    // SDAPI
    public func reportError(_ error: Error, forUniqueClientId uniqueClientId: String, os: Optional<String>, clientVersion: Optional<String>, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
        
            let description = error.localizedDescription
            var context = ""
            var s_error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_report_error(clientVersion, os, uniqueClientId, description, context, &s_error)
            defer {
                if res == -1 {
                    sddk_free_error(&s_error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: s_error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    
    // gc
    
    public func gc(completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_gc(self.state!, &error)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    // keychain
    
    public func getKeychainItem(withUser username: String, service: String) throws -> String {
        var secret_ptr: UnsafeMutablePointer<CChar>? = nil
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_get_keychain_item(username, service, &secret_ptr, &error)
        defer {
            if res >= 0 {
                sddk_free_string(&secret_ptr)
            }
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case -1:
            let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
            throw e
        default:
            let secret = String(cString: secret_ptr!)
            return secret
        }
    }
    
    public func setKeychainItem(withUser username: String, service: String, secret: String) throws {
        
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        let res = sddk_set_keychain_item(username, service, secret, &error)
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case -1:
            let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
            throw e
        default:
            return
        }
    }
    
    
    public func deleteKeychainItem(withUser username: String, service: String) throws {
        
        var error: UnsafeMutablePointer<SDDKError>? = nil
        
        
        let res = sddk_delete_keychain_item(username, service, &error)
        defer {
            if res == -1 {
                sddk_free_error(&error)
            }
        }
        switch res {
        case -1:
            let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
            throw e
        default:
            return
        }
    }
    
    // remote fs
    
    public func remoteFSCreateDirectory(path: String, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_remote_mkdir(self.state!, &error, path)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func remoteFSDeleteDirectory(path: String, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_remote_rmdir(self.state!, &error, path)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
    
    public func remoteFSMoveDirectory(path: String, newPath: String, completionQueue queue: DispatchQueue, success: @escaping SDKSuccess, failure: @escaping SDKFailure) {
        
        DispatchQueue.global(priority: .default).async {
            var error: UnsafeMutablePointer<SDDKError>? = nil
            
            let res = sddk_remote_mv(self.state!, &error, path, newPath)
            defer {
                if res == -1 {
                    sddk_free_error(&error)
                }
            }
            switch res {
            case 0:
                queue.async { success() }
            default:
                let e = SDKErrorFromSDDKError(sdkError: error!.pointee)
                queue.async { failure(e) }
            }
        }
    }
}

