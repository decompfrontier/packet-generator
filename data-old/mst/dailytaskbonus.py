from schema import *

@keyjson(key_group = "p283g07d", array=ArrayStep.Single)
class DailyTaskBonusMst:
    bonus_brave_points = { "k3bD738b": intstr }

@keyjson(key_group = "k23D7d43", array=ArrayStep.Array)
class DailyTaskMst:
    key = { "O36Qv37k": str } # TODO: make this an enum
    title = { "hd2Jf3nC": str }
    desc = { "M7yKr4c1": str }
    task_count = { "Y3DbX5ot": intstr }
    # TODO: better names for this
    task_brave_pts = { "T4bV8aI9": intstr }
    brave_points_total = { "bya9a67k": intstr }
    brave_points = { "22rqpZTo": intstr }
    area_id = { "a3011F8b": str }
    times_completed = { "9cKyb15U": int, "doc": "Number of times the task was completed" }

@keyjson(key_group = "a739yK18", array=ArrayStep.Array)
class DailyTaskPrizeMst:
    id = { "d83aQ39U": intstr }
    title = { "T091Rsbe": str }
    desc = { "L2VkgH08": str }
    present_type = { "30Kw4WBa": intstr }
    reward_id = { "TdDHf59J": intstr }
    reward_count = { "wJsB35iH": intstr }
    reward_param = { "37moriMq": str }
    brave_points_cost = { "4NuIwm77": intstr }
    time_limit = { "qY49LBjw": long }
    max_claim_count = { "D2BlS89M": intstr } # max number of times you can claim this
    current_claim_count = { "jT3oB57e": int } # if the user has claimed it
    milestone_prize = { "J3l5We66": strbool }
