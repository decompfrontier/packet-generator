from enum import Enum, Flag
from schema import *

"""This module contains the basic game request/response."""

@processable
class GmeErrorFlags(Flag):
    """Flags of the error happend in the game."""
    
    __doc_fields__ = {
        0: "No error oncurred.",
        1: "The server oncurred an error.",
        2: "The custom command should be handled."
    }

    NoError = 0
    IsInError = 1
    ShouldHandleCommands = 2

@processable
class GmeErrorCommand(Enum):
    "Type of command to do after the user presses OK."

    __doc_fields__ = {}

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
    """Header of a game request/response."""
    
    id = { "Hhgi79M1": str, "doc": "ID of the request" }
    client_id = { "aV6cLn3v": str, "doc": "ID of the client that invoked the request" }

@keyjson(key_group = "a3vSYuq2", array = False)
class GmeBody:
    """Content of the game request/response."""

    body = { "Kn51uR4Y": str, "doc": "Encrypted JSON content." }

@keyjson(key_group = "b5PH6mZa", array = False)
class GmeError:
    """Object that stores any possible error with the request or response."""

    flag = { "3e9aGpus": GmeErrorFlags, "doc": "Error flags." }
    cmd = { "iPD12YCr": GmeErrorCommand, "doc": "Action to execute after pressing OK. (Only enabled if the flag `ShouldHandleCommands` is set)" }
    message = { "ZC0msu2L": str, "doc": "Message to show on the error." }
    url = { "zcJeTx18": str, "doc": "URL to open in the browser after OK is pressed. (like for update the game)" }

@json(array = False)
class GmeAction:
    "Main packet of interaction between client and server."
    
    header = { "" : GmeHeader, "doc": "Header of the message." }
    body = { "" : GmeBody, "default": DefaultType.Omit, "doc": "Body of the message." }
    error = { "": GmeError, "default": DefaultType.Omit, "doc": "Error object in case of an error." }
