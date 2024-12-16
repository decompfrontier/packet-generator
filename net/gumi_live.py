from enum import Enum
from schema import *

# API V3 (DLS API) api-sl.bfww.gumi.sg

class StatusEnum(Enum):
    Success = "successful"
    Error = "failed"

@json(array = False)
class GuestLogin:
    status = { "status": StatusEnum }
    token = { "token": str }
    user_id = { "game_user_id": int }
    status_number = { "status_no": int }

@json(array = False)
class GameDls:
    game_ip = { "game": str }
    resource_ip = { "resource": str }
    version = { "mstv": str }
    gumilive_ip = { "gumilive": str }
    bg_image = { "bgimage": str }

@json(array = False)
class SREE:
    body = { "SREE": str } # crypted gumilive api data
