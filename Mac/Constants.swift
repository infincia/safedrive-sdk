//
//  Constants.swift
//  SafeDriveSDK
//
//  Created by steve on 3/21/17.
//  Copyright © 2017 SafeDrive. All rights reserved.
//

import Foundation

public let SDErrorDomainNotReported = "io.safedrive.notreported"
public let SDErrorDomainReported = "io.safedrive.reported"
public let SDErrorDomainInternal = "io.safedrive.internal"

public enum SDKLogLevel: UInt8 {
    case error = 0
    case warn = 1
    case info = 2
    case debug = 3
    case trace = 4
}
