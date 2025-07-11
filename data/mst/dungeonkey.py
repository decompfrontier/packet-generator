from schema import *

@keyjson(key_group = "4NG79sX1", array=ArrayStep.Array)
class DungeonKeyMst:
    id = { "16KMNJLb": intstr }
    name = { "BM29ZgnK": str }
    dungeon_id = { "MHx05sXt": intstr }
    thumbnail_img = { "M2cv6dum": str }
    open_img = { "VX0j1fni": str }
    close_img = { "9unNZ6b0": str }
    limit_sec = { "i9sBW8uD": intstr }
    possession_limit = { "N7I9vYZb": str }
    distribute_count = { "khsb74Nq": int, "default": DefaultType.Omit }
    distribute_flag = { "EK5I6MQ9": intstr } # TODO: is this a boolean?
    usage_pattern = { "CR6aKWg8": str }
    state = { "j0Uszek2": intstr }

