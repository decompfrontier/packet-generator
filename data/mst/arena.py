from schema import *

@keyjson(key_group = "6kWq78zx")
class ArenaRankMst:
    """Rankings of the PVP Arena."""

    id = { "JmFn3g9t": int, "doc": "Ranking ID" }
    name = { "rGm09bav": str, "doc": "Ranking name" }
    rank_point_start = { "w0aTd94Y": int, "doc": "Points when this rank starts (included)" }
    rank_point_end = { "1U3eBCyY": int, "doc": "Points when this rank ends (included)" }
    reward_type = { "IkmC8gG2": int, "doc": "Type of reward" }
    reward_param = { "empaR60j": str }
    scenario_info = { "N4XVE1uA": str }
