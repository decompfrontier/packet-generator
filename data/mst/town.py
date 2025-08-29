from schema import *

@keyjson(key_group = "d0EkJ4TB", array=ArrayStep.Array)
class TownFacilityLvMst:
    id = { "y9ET7Aub": int }
    lv = { "D9wXQI2V": int }
    karma = { "HTVh8a65": int }
    release_receipe = { "rGoJ6Ty9": commalist[int] }

@keyjson(key_group = "9ekQ4tZq", array=ArrayStep.Array)
class TownLocationLvMst:
    id = { "y9ET7Aub": intstr }
    lv = { "D9wXQI2V": intstr }
    karma = { "HTVh8a65": intstr }
    release_receipe = { "rGoJ6Ty9": commalist[int] }

@keyjson(key_group = "Lh1I3dGo", array=ArrayStep.Array)
class TownFacilityMst:
    id = { "y9ET7Aub": intstr }
    name = { "aAFI6S5w": str }
    pos_x = { "SnNtTh51": intstr }
    pos_y = { "M6C1aXfR": intstr }
    width = { "dRhvW13q": intstr }
    height = { "FCzW4g6P": intstr }
    need_mission_id = { "HSRhkf70": intstr }

@keyjson(key_group = "1y2JDv79", array=ArrayStep.Array)
class TownLocationMst:
    id = { "un80kW9Y": intstr }
    name = { "6ntp4rMV": str }
    pos_x = { "SnNtTh51": intstr }
    pos_y = { "M6C1aXfR": intstr }
    width = { "dRhvW13q": intstr }
    height = { "FCzW4g6P": intstr }
    need_mission_id = { "HSRhkf70": intstr }
    effect_type = { "jeR2rN3V": intstr }
