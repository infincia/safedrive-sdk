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
    
    public init(sdkError: SDDKError) {
        self.message = String(cString: sdkError.message!)
        guard let type = SDKErrorType(rawValue: Int(sdkError.error_type.rawValue)) else {
            fatalError("no error type for \(sdkError.error_type)")
        }
        
        self.kind = type
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
            return SDKErrorDomainInternal
        case .Internal:
            return SDKErrorDomainInternal
        case .RequestFailure:
            return SDKErrorDomainNotReported
        case .NetworkFailure:
            return SDKErrorDomainNotReported
        case .Conflict:
            return SDKErrorDomainNotReported
        case .BlockMissing:
            return SDKErrorDomainReported
        case .SessionMissing:
            return SDKErrorDomainInternal
        case .RecoveryPhraseIncorrect:
            return SDKErrorDomainNotReported
        case .InsufficientFreeSpace:
            return SDKErrorDomainNotReported
        case .Authentication:
            return SDKErrorDomainNotReported
        case .UnicodeError:
            return SDKErrorDomainReported
        case .TokenExpired:
            return SDKErrorDomainNotReported
        case .CryptoError:
            return SDKErrorDomainReported
        case .IO:
            return SDKErrorDomainNotReported
        case .SyncAlreadyInProgress:
            return SDKErrorDomainNotReported
        case .RestoreAlreadyInProgress:
            return SDKErrorDomainNotReported
        case .ExceededRetries:
            return SDKErrorDomainNotReported
        case .KeychainError:
            return SDKErrorDomainReported
        case .BlockUnreadable:
            return SDKErrorDomainReported
        case .SessionUnreadable:
            return SDKErrorDomainReported
        case .ServiceUnavailable:
            return SDKErrorDomainReported
        case .Cancelled:
            return SDKErrorDomainNotReported
        case .FolderMissing:
            return SDKErrorDomainNotReported
        case .KeyCorrupted:
            return SDKErrorDomainReported
        }
    }
    
    public var errorCode: Int {
        return self.kind.rawValue
    }
}
