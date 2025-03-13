from enum import Enum, Flag
from schema import *

class GmeErrorFlags(Flag):
    doc = "Flags of the errors happend in the game."
    
    doc_f = {
        0: "No error oncurred.",
        1: "The server oncurred an error.",
        2: "The custom command should be handled."
    }

    NoError = 0
    IsInError = 1
    ShouldHandleCommands = 2

class GmeErrorCommand(Enum):
    doc = "Type of command to do after the user presses OK."

    doc_f = {}

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
    flag = { "3e9aGpus": GmeErrorFlags, "doc": "Error flags." }
    cmd = { "iPD12YCr": GmeErrorCommand, "doc": "Action to execute after pressing OK. (Only enabled if the flag `ShouldHandleCommands` is set)" }
    message = { "ZC0msu2L": str, "doc": "Message to show on the error." }
    url = { "zcJeTx18": str, "doc": "URL to open in the browser after OK is pressed. (like for update the game)" }

@json(array = False)
class GmeAction:
    doc = "Main packet of interaction between client and server."
    
    header = { "" : GmeHeader, "doc": "Header of the message." }
    body = { "" : GmeBody, "omit_on_default": True, "doc": "Body of the message." }
    error = { "": GmeError, "omit_on_default": True, "doc": "Error object in case of an error." }
