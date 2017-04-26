//
//  SafeDriveSDK.swift
//  SafeDriveSDK
//
//  Created by steve on 5/27/16.
//  Copyright © 2016 SafeDrive. All rights reserved.
//

import XCTest

@testable import SafeDriveSDK

class SafeDriveSDKTests: XCTestCase {

    let storage_path = "/Users/steve/Library/Application Support/SafeDrive/"
    
    let user = "stephen@safedrive.io"
    
    var sdk = SafeDriveSDK.sharedSDK

    override class func setUp() {
        super.setUp()
    }

    override func setUp() {
        super.setUp()
        if let uniqueClientId = self.sdk.uniqueClientID(self.user) {
            self.sdk.setUp(local_storage_path: self.storage_path, unique_client_id: uniqueClientId)
        }
    }

    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
        super.tearDown()
    }

    func testAddSyncFolder() {
        try! self.sdk.addFolder("Downloads", path: "/Users/steve/Downloads")
    }

    func testListSyncFolders() {
        let syncFolders = try! self.sdk.getFolders()
        let _ = syncFolders.map { (folder) -> Folder in
            return folder
        }
    }

    func testListBackupsForFolder() {
        let sessions = try! self.sdk.getSessions()
        let _ = sessions.map { (session) -> SyncSession in
            Swift.print("Session: \(session.name)")
            return session
        }
    }

    func testCreateBackupForFolder() {
        let folders = try! self.sdk.getFolders()
        
        let syncFolders = folders.filter { (folder) -> Bool in
            folder.name == "Downloads"
        }
        let syncFolder = syncFolders.first!
        
        self.sdk.syncFolder(folderID: syncFolder.id, progress: { (total, current, progress) in
            //
        }, success: {
            //
        }, failure: { (error: Error) in
            print("Backup failed: \(error.localizedDescription)")
        })
    }

}

