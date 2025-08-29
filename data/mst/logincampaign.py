from schema import *

@keyjson(key_group = "Bd29Pqw0")
class LoginCampaignMst:
    id = { "H1Dkq93v": intstr }
    start_date = { "qA7M9EjP": datetimeunix }
    total_days = { "1adb38d5": intstr }
    image = { "b38adb8i": str }

@keyjson(key_group = "bD18x9Ti", array=ArrayStep.Array)
class LoginCampaignReward:
    id = { "H1Dkq93v": intstr }
    reward_day = { "n0He37p1": intstr }
    reward_img = { "b38adb8i": str }
