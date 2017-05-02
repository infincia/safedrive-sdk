//
//  Errors.swift
//  SafeDriveSDK
//
//  Created by steve on 5/1/17.
//  Copyright © 2017 SafeDrive. All rights reserved.
//

import Foundation

import SDDK

public enum SDKErrorType: Int {
    case StateMissing = 0x0000
    case Internal = 0x0001
    case RequestFailure = 0x0002
    case NetworkFailure = 0x0003
    case Conflict = 0x0004
    case BlockMissing = 0x0005
    case SessionMissing = 0x0006
    case RecoveryPhraseIncorrect = 0x0007
    case InsufficientFreeSpace = 0x0008
    case Authentication = 0x0009
    case UnicodeError = 0x000A
    case TokenExpired = 0x000B
    case CryptoError = 0x000C
    case IO = 0x000D
    case SyncAlreadyInProgress = 0x000E
    case RestoreAlreadyInProgress = 0x000F
    case ExceededRetries = 0x0010
    case KeychainError = 0x0011
    case BlockUnreadable = 0x0012
    case SessionUnreadable = 0x0013
    case ServiceUnavailable = 0x0014
    case Cancelled = 0x0015
    case FolderMissing = 0x0016
    case KeyCorrupted = 0x0017
    
}

public struct SDKError {
    public var message: String
    public var kind: SDKErrorType
    
    public var code: Int {
        return self.kind.rawValue
    }
    
    public init(message: String, kind: SDKErrorType) {
        self.message = message
        self.kind = kind
    }
}

extension SDKError: LocalizedError {
    public var errorDescription: String? {
        get {
            return self.message
        }
    }
}

extension SDKError:  CustomNSError {
    var errorDomain: String {
        switch self.kind {
        case .StateMissing:
            return SDErrorDomainInternal
        case .Internal:
            return SDErrorDomainInternal
        case .RequestFailure:
            return SDErrorDomainNotReported
        case .NetworkFailure:
            return SDErrorDomainNotReported
        case .Conflict:
            return SDErrorDomainNotReported
        case .BlockMissing:
            return SDErrorDomainReported
        case .SessionMissing:
            return SDErrorDomainInternal
        case .RecoveryPhraseIncorrect:
            return SDErrorDomainNotReported
        case .InsufficientFreeSpace:
            return SDErrorDomainNotReported
        case .Authentication:
            return SDErrorDomainNotReported
        case .UnicodeError:
            return SDErrorDomainReported
        case .TokenExpired:
            return SDErrorDomainNotReported
        case .CryptoError:
            return SDErrorDomainReported
        case .IO:
            return SDErrorDomainNotReported
        case .SyncAlreadyInProgress:
            return SDErrorDomainNotReported
        case .RestoreAlreadyInProgress:
            return SDErrorDomainNotReported
        case .ExceededRetries:
            return SDErrorDomainNotReported
        case .KeychainError:
            return SDErrorDomainReported
        case .BlockUnreadable:
            return SDErrorDomainReported
        case .SessionUnreadable:
            return SDErrorDomainReported
        case .ServiceUnavailable:
            return SDErrorDomainReported
        case .Cancelled:
            return SDErrorDomainNotReported
        case .FolderMissing:
            return SDErrorDomainNotReported
        case .KeyCorrupted:
            return SDErrorDomainReported
        }
    }
    
    public var errorCode: Int {
        return self.kind.rawValue
    }
}




func SDKErrorFromSDDKError(sdkError: SDDKError) -> SDKError {
    let s = String(cString: sdkError.message!)
    guard let type = SDKErrorType(rawValue: Int(sdkError.error_type.rawValue)) else {
        fatalError("no error type for \(sdkError.error_type)")
    }
    
    return SDKError(message: s, kind: type)
}
