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

    public func setUp(local_storage_path: String, unique_client_id: String) {
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
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
        }
        let res = sddk_login(state, username, password)
        
        switch res {
        case 0:
            return
        default:
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
    }

    public func loadKeys(_ recoveryPhrase: String?, storePhrase: @escaping SaveRecoveryPhrase) throws {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
        }
  
        let res = sddk_load_keys(unsafeBitCast(storePhrase, to: UnsafeMutableRawPointer.self), state, recoveryPhrase) { (context, phrase) in
            // call back to Swift to save the phrase somewhere
            let b = unsafeBitCast(context, to: SaveRecoveryPhrase.self)
            let p = String(cString: phrase!)
            var m = phrase
            b(p)
            sddk_free_string(&m)
        }
        
        switch res {
        case 0:
            return
        default:
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
    }
    
    public func uniqueClientID(_ email_address: String) throws -> Optional<String> {
        var unique_client_id: UnsafeMutablePointer<CChar>? = nil
        let res = sddk_get_unique_client_id(email_address, &unique_client_id)
        defer {
            if res >= 0 {
                sddk_free_string(&unique_client_id)
            }
        }
        if res < 0 {
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
        switch res {
        case 0:
            return String(cString: unique_client_id!)
        default:
            return nil
        }
    }

    public func addFolder(_ name: String, path: String) throws {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)

        }
        let res = sddk_add_sync_folder(state, name, path)
        switch res {
        case 0:
            return
        default:
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
    }
    
    public func removeFolder(_ folderId: UInt32) throws {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)

        }
        let res = sddk_remove_sync_folder(state, folderId)
        switch res {
        case 0:
            return
        default:
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
    }
    
    public func getFolder(folderId: UInt32) throws -> Folder {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
        }
        var folder_ptr: UnsafeMutablePointer<SDDKFolder>? = nil
        let res = sddk_get_sync_folder(state, folderId, &folder_ptr)
        defer {
            if res >= 0 {
                sddk_free_folders(&folder_ptr, UInt64(res))
            }
        }
        if res < 0 {
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }

        let name = String(cString: folder_ptr!.pointee.name)
        let path = String(cString: folder_ptr!.pointee.path)
        let id = folder_ptr!.pointee.id
        let folder = Folder(id: id, name: name, path: path)
        
        return folder
    }

    public func getFolders() throws -> [Folder] {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
        }
        var folder_ptr: UnsafeMutablePointer<SDDKFolder>? = nil
        let res = sddk_get_sync_folders(state, &folder_ptr)
        defer {
            if res >= 0 {
                sddk_free_folders(&folder_ptr, UInt64(res))
            }
        }
        if res < 0 {
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
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
    }
    
    public func getSessions() throws -> [SyncSession] {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
        }
        var sessions_ptr: UnsafeMutablePointer<SDDKSyncSession>? = nil
        let res = sddk_get_sync_sessions(state, &sessions_ptr)
        defer {
            if res >= 0 {
                sddk_free_sync_sessions(&sessions_ptr, UInt64(res))
            }
        }
        if res < 0 {
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
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
    }
    
    public func syncFolder(folderID: UInt32, sessionName: String, progress: @escaping SyncSessionProgress, success: @escaping SyncSessionSuccess, failure: @escaping SyncSessionFailure) {
        guard let state = self.state else {
            let e = NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
            failure(e)
            return
        }
        DispatchQueue.global(priority: .default).async {
            let result = sddk_sync(unsafeBitCast(progress, to: UnsafeMutableRawPointer.self), state, sessionName, folderID, { (context, total, current, percent, tick) in
                // call back to Swift to report progress
            
                let b = unsafeBitCast(context, to: SyncSessionProgress.self)
                b(total, current, percent)
            })
            switch result {
            case 0:
                success()
            default:
                let e = NSError(domain: "io.safedrive.sdk", code: Int(result), userInfo: nil)
                failure(e)
            }
        }

    }
    
    public func restoreFolder(folderID: UInt32, sessionName: String, destination: URL, progress: @escaping SyncSessionProgress, success: @escaping SyncSessionSuccess, failure: @escaping SyncSessionFailure) {
        guard let state = self.state else {
            let e = NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)
            failure(e)
            return
        }
        DispatchQueue.global(priority: .default).async {
            let result = sddk_restore(unsafeBitCast(progress, to: UnsafeMutableRawPointer.self), state, sessionName, folderID, destination.path, { (context, total, current, percent, tick) in
                // call back to Swift to report progress
            
                let b = unsafeBitCast(context, to: SyncSessionProgress.self)
                b(total, current, percent)
            })
            switch result {
            case 0:
                success()
            default:
                let e = NSError(domain: "io.safedrive.sdk", code: Int(result), userInfo: nil)
                failure(e)
            }
        }

    }
    
    
    
    
    // gc
    
    public func gc() throws {
        guard let state = self.state else {
            throw NSError(domain: "io.safedrive.sdk", code: -9000, userInfo: nil)

        }
        let res = sddk_gc(state)
        switch res {
        case 0:
            return
        default:
            throw NSError(domain: "io.safedrive.sdk", code: Int(res), userInfo: nil)
        }
    }
    
}

