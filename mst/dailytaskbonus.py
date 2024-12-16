from schema import *

@keyjson(key_group = "p283g07d", single = True)
class DailyTaskBonusMst:
    bonus_brave_points = { "k3bD738b": int }

@keyjson(key_group = "k23D7d43")
class DailyTaskMst:
    key = { "O36Qv37k": str } # TODO: make this an enum
    title = { "hd2Jf3nC": str }
    desc = { "M7yKr4c1": str }
    task_count = { "Y3DbX5ot": int }
    # TODO: better names for this
    task_brave_pts = { "T4bV8aI9": int }
    brave_points_total = { "bya9a67k": int }
    brave_points = { "22rqpZTo": int }
    area_id = { "a3011F8b": str }
    times_completed = { "9cKyb15U": int }

@keyjson(key_group = "a739yK18")
class DailyTaskPrizeMst:
    id = { "d83aQ39U": int }
    title = { "T091Rsbe": str }
    desc = { "L2VkgH08": str }
    present_type = { "30Kw4WBa": int }
    reward_id = { "TdDHf59J": int }
    reward_count = { "wJsB35iH": int }
    reward_param = { "37moriMq": str }
    brave_points_cost = { "4NuIwm77": int }
    time_limit = { "qY49LBjw": long }
    max_claim_count = { "D2BlS89M": int } # max number of times you can claim this
    current_claim_count = { "jT3oB57e": int } # if the user has claimed it
    milestone_prize = { "J3l5We66": strbool }
