from enum import Enum
from schema import *

class GmeErrorID(Enum):
    NoError = 0
    HaveError = 1
    ErrorWithForceClose = 3

class GmeErrorOperation(Enum):
    Retry = 2
    Continue = 3
    Close = 4
    Close2 = 5
    ReturnToGame = 6
    RaidError = 7
    Continue2 = 8
    LogoutFacebook = 9
    Close3 = 10

@keyjson(key_group = "F4q6i9xe", array = False)
class GmeHeader:
    id = { "Hhgi79M1": str }
    client_id = { "aV6cLn3v": str }

@keyjson(key_group = "a3vSYuq2", array = False)
class GmeBody:
    body = { "Kn51uR4Y": str }

@keyjson(key_group = "b5PH6mZa", array = False)
class GmeError:
    id = { "3e9aGpus": GmeErrorID }
    continue_id = { "iPD12YCr": GmeErrorOperation }
    message = { "ZC0msu2L": str }
    unk1 = { "zcJeTx18": str }

@json(array = False)
class GmeAction:
    header = { "" : GmeHeader }
    body = { "" : GmeBody, "omit_on_default": True }
    error = { "": GmeError, "omit_on_default": True }
