//
//  SafeDriveSDK.swift
//  SafeDriveSDK
//
//  Created by steve on 8/23/16.
//  Copyright © 2016 SafeDrive. All rights reserved.
//

import Foundation

import sddk

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

enum SDKError: Error {
    case StateMissing(message: String)
    case Internal(message: String)
    case RequestFailure(message: String)
    case NetworkFailure(message: String)
    case Conflict(message: String)
    case BlockMissing(message: String)
    case SessionMissing(message: String)
    case RecoveryPhraseIncorrect(message: String)
    case InsufficientFreeSpace(message: String)
    case Authentication(message: String)
    case UnicodeError(message: String)
    case TokenExpired(message: String)
    case CryptoError(message: String)
    case IO(message: String)
    case SyncAlreadyInProgress(message: String)
    case RestoreAlreadyInProgress(message: String)
}

func SDKErrorFromSDDKError(sdkError: SDDKError) -> SDKError {
    let s = String(cString: sdkError.message!)
    var e: SDKError
    
    switch sdkError.error_type.rawValue {
    case 0x0001:
        e = SDKError.Internal(message: s)
    case 0x0002:
        e = SDKError.RequestFailure(message: s)
    case 0x0003:
            e = SDKError.NetworkFailure(message: s)
    case 0x0004:
            e = SDKError.Conflict(message: s)
    case 0x0005:
            e = SDKError.BlockMissing(message: s)
    case 0x0006:
            e = SDKError.SessionMissing(message: s)
    case 0x0007:
            e = SDKError.RecoveryPhraseIncorrect(message: s)
    case 0x0008:
            e = SDKError.InsufficientFreeSpace(message: s)
    case 0x0009:
            e = SDKError.Authentication(message: s)
    case 0x000A:
            e = SDKError.UnicodeError(message: s)
    case 0x000B:
            e = SDKError.TokenExpired(message: s)
    case 0x000C:
            e = SDKError.CryptoError(message: s)
    case 0x000D:
            e = SDKError.IO(message: s)
    case 0x000E:
            e = SDKError.SyncAlreadyInProgress(message: s)
    case 0x000F:
            e = SDKError.RestoreAlreadyInProgress(message: s)
    default:
        exit(1)
        break
    }
    return e
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

    public func setUp(local_storage_path: String, unique_client_id: String) throws {
        var config: SDDKConfiguration
        #if DEBUG
        config = SDDKConfigurationStaging
        #else
        config = SDDKConfigurationProduction
        #endif

        self.state = sddk_initialize(local_storage_path, unique_client_id, config)
    }
    
    deinit {
        sddk_free_state(&state)
    }
    
    public func login(_ username: String, password: String) throws {
        guard let state = self.state else {
            throw SDKError.StateMissing(message: "State missing, cannot continue")
        }
        var error: UnsafeMutablePointer<SDDKError>? = nil

        let res = sddk_login(state, username, password, &error)
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

    public func loadKeys(_ recoveryPhrase: String?, storePhrase: @escaping SaveRecoveryPhrase) throws {
        guard let state = self.state else {
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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
            let e = NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
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
            let e = NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
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
            throw SDKError.StateMissing(message: "State missing, cannot continue")
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

