from schema import *

class ArenaRewardTypes:
    # ???
    _1 = "1"
    _2 = "2"


@keyjson(key_group = "6kWq78zx", array=ArrayStep.Array)
class ArenaRankMst: # F_ARENA_MST
    """Rankings of the PVP Arena."""

    id = { "JmFn3g9t": intstr, "doc": "Ranking ID" }
    name = { "rGm09bav": str, "doc": "Ranking name" }
    rank_point_start = { "w0aTd94Y": intstr, "doc": "Points when this rank starts (included)" }
    rank_point_end = { "1U3eBCyY": intstr, "doc": "Points when this rank ends (included)" }
    reward_type = { "IkmC8gG2": intstr, "doc": "Type of reward" } # TODO: discover and make an Enum with str-types !!
    reward_param = { "empaR60j": str, "doc": "Configuration of the reward" } # list[int]: ArrayType.DOUBLEDOT
    scenario_info = { "N4XVE1uA": str, "doc": "ID of the scene to play" }
