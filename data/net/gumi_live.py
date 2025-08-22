from enum import Enum
from schema import *

"""This module contains the data for the gumi live api """

# API V3 (DLS API): api-sl.bfww.gumi.sg

@processable
class StatusEnum(Enum):
    """Result of the login status"""

    __doc_field__ = {
        "successful": "Operation completed successfully",
        "failed": "Operation failed"
    }

    Success = "successful"
    Error = "failed"
    
@processable
class PlatformEnum(Enum):
    """Platforms that runs the game"""
    
    __doc_field__ = {
        "Android": "Android platform",
        "Windows": "Windows platform"
    }
    
    Android = "Android",
    Windows = "Windows"


@json
class GuestLogin:
    """Gumi API login"""

    status = { "status": StatusEnum, "doc": "Status of the operation" }
    token = { "token": str, "doc": "Login token" }
    user_id = { "game_user_id": str, "doc": "ID of the game to perform the login" }
    status_number = { "status_no": intstr, "doc": "ID of the error" } # TODO: make an enum out of this
    #servers = { "servers": list[str], "doc": "Unknown" } # TODO: this is a list of IP?

# TODO: PHP something
class GuestLoginReq:
    """Gumi API login request"""

    deviceModel = { "dn": str, "doc": "Device model" }
    devicePlatform = { "dp": PlatformEnum, "doc": "Device platform" }
    deviceAdId = { "vid": str, "doc": "Device Advertising ID" }
    deviceVersion = { "v": str, "doc": "Device version" }
    altVid = { "altvid": str }
    ak = { "ak": str }
    identifiers = { "identifiers": str }

@json
class GameDls:
    """Game dynamic configuration"""

    game_ip = { "game": str, "doc": "Game server address" }
    resource_ip = { "resource": str, "doc": "CDN server address where resources will be downloaded" }
    version = { "mstv": intstr, "doc": "Game version" }
    gumilive_ip = { "gumilive": str, "doc": "Gumi live API login server address" }
    bg_image = { "bgimage": str, "doc": "Dynamic background image to show during login" }
    force = { "force": bool, "default": DefaultType.Omit, "doc": "Block the client login attempt and force it to close" }
    msg = { "force_msg": str, "default": DefaultType.Omit, "doc": "Message to show when the login attempt was blocked (only valid when force is true)"}

@json
class SREE:
    """This object is a container of encrypted JSON data used in DLS API.
    
    The content of the SREE is encrypted with AES/CBC (No padding) and encoded with Base64.
    
    AES Key: 7410958164354871
    AES IV: Bfw4encrypedPass"""

    body = { "SREE": str, "doc": "Crypted data" }

#@processable
class GumiLiveInfo:
    pass
