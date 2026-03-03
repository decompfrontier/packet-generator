from schema import *
from datetime import datetime

@keyjson(key_group = "Pf97SzVw", array=ArrayStep.Array)
class GachaEffectMst:
    id = { "u0vkt9yH": intstr }
    gatcha_id = { "7Ffmi96v": intstr }
    rare = { "7ofj5xa1": intstr }
    rate = { "ug9xV4Fz": floatstr }
    effect_before = { "7ZNcmYS2": str }
    effect_after = { "tj0i9JhC": str }
    effect = { "YTx3c1jQ": str }
    
@keyjson(key_group = "IBs49NiH", array=ArrayStep.Array)
class GachaCategory:
    id = { "vx9uyQVQ": int }
    img = { "In7lGGLn": str }
    disp_order = { "2r4EoNt4": int }
    gatcha_id_list = { "3rCmq58M": str }
    start_date = { "qA7M9EjP": datetimeunix }
    end_date = { "SzV0Nps7": datetimeunix }

@keyjson(key_group = "5Y4GJeo3", array=ArrayStep.Array)
class GachaMst:
    id = { "7Ffmi96v": intstr }
    name = { "4N27mkt1": str }
    type = { "S1oz60Hc": intstr } # TODO: an enum?
    priority = { "yu18xScw": intstr }
    start_date = { "qA7M9EjP": datetime } # (2017-10-24 08:00:00)
    end_date = { "SzV0Nps7": datetime } # (2017-10-24 08:00:00)
    start_hour = { "2HY3jpgu": str }
    end_hour = { "v9TR3cDz": str }
    need_friend_point = { "J3stQ7jd": intstr }
    need_gems = { "03UGMHxF": intstr }
    once_day_flag = { "4tswNoV9": strbool }
    bg_img = { "1Dg0vUX3": str }
    btn_img = { "W9ABuJj2": str }
    door_img = { "uKYf13AH": str }
    caption_msg = { "3sdHQb69": str }
    detail_msg = { "W2c9g0Je": str }
    comment_msg = { "gVSj32QH": str }
    gatcha_group_id = { "TCnm1F4v": intstr }
    description = { "qp37xTDh": str }
    gatcha_detail_id = { "8Z9CYQDq": str, "default": DefaultType.Omit }
    contents_banner_img = { "sA9dDAqB": str, "default": DefaultType.Omit }

'''@keyjson(key_group = "da3qD39b")
class ResummonGachaMst:
    pass

@keyjson(key_group = "hE1d083b")
class SummonTicketV2Mst:
    pass'''
