from schema import *

@keyjson(key_group = "Bd29Pqw0")
class LoginCampaignMst:
    id = { "H1Dkq93v": int }
    start_date = { "qA7M9EjP": datetimeunix }
    total_days = { "1adb38d5": int }
    image = { "b38adb8i": str }

@keyjson(key_group = "bD18x9Ti", array=ArrayStep.Array)
class LoginCampaignReward:
    id = { "H1Dkq93v": int }
    reward_day = { "n0He37p1": int }
    reward_img = { "b38adb8i": str }
