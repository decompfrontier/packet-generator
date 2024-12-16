from schema import *

@keyjson(key_group = "4NG79sX1")
# TODO: is this configurable?
class DungeonKeyMST:
    id = { "16KMNJLb": int }
    name = { "BM29ZgnK": str }
    dungeon_id = { "MHx05sXt": int }
    thumbnail_img = { "M2cv6dum": str }
    open_img = { "VX0j1fni": str }
    close_img = { "9unNZ6b0": str }
    limit_sec = { "i9sBW8uD": int }
    possession_limit = { "N7I9vYZb": str }
    distribute_count = { "khsb74Nq": int, "omit_on_default": True }
    distribute_flag = { "EK5I6MQ9": int } # TODO: is this a boolean?
    usage_pattern = { "CR6aKWg8": str }
    state = { "j0Uszek2": int }
