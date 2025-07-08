from schema import *

@processable
class BannerOperativeSystem(Enum):
    """Types of operative systems"""

    __doc_field__ = {
        "Android": "Android system"
    }

    # TODO: discover the others
    Android = "2"

@keyjson(key_group = "Pk5F1vhx", array=ArrayStep.Array)
class BannerInfoMst:
    """Banner configuration in the links page"""

    id = { "oL71Fz3a": intstr, "doc": "ID of the banner" }
    name = { "NyYKc1A5": str, "doc": "Internal name of the banner" }
    target_os = { "aL70hVYQ": commalist[BannerOperativeSystem], "doc": "List of operative systems allowed" }
    display_order = { "XuJL4pc5": intstr, "doc": "Order which to display the banner" }
    url = { "jsRoN50z": str }
    image = { "1gDkL6XR": int, "doc": "URL of the image to display" }
    param = { "t5R47iwj": str }
    page_type = { "LM34kfVC": str, "doc": "This parameter seems unused in the last version of the game" }
    read_count = { "d36D1g8T": intstr, "doc": "Number of times the page was read" }
