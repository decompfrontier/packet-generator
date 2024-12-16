from schema import *

@json(array = False)
class SlotGameInfo:
    id = { "zS45RFGb": int }
    name = { "I1Cki7Pb": str }
    reel_pos = { "h1PSnk5t": str }
    use_medal = { "b5yeVr61": str }
    slot_help_url = { "jsRoN50z": str }
    slot_image = { "TX98PnpE": str }

@json
class SlotGamePictureInfo:
    id = { "sE6tyI9i": int }
    picture_name = { "iQM9dH0F": str }

@keyjson(key_group = "C38FmiUn", array = False)
class SlotGameInfoR:
    info = { "C38FmiUn": SlotGameInfo }
    pictures = { "rY6j0Jvs": SlotGamePictureInfo }

@json
class SlotGameReelInfo:
    id = { "PINm2pM5": int }
    reel_data = { "Z8eJi4pq": str }

@keyjson(key_group = "j129kD6r")
class VideoAdInfo:
    id = { "k3ab6D82": int }
    available_value = { "Diwl3b56": int }
    region_id = { "Y3de0n2p": int }
    video_enabled = { "26adZ1iy": intbool }
    next_available_time_left = { "oohpPLCt": long }

@keyjson(key_group = "bpD29eiQ")
class VideoAdRegion:
    id = { "k3ab6D82": int }
    country_code = { "j3d6E2ia": str }

@json(array = False)
class VideoAdsSlotGameStandInfo:
    ads_count = { "wRIgGCHh": int }
    max_ads_count = { "JwBrVzIZ": int }
    current_bouns = { "BrMgnCaT": int }
    max_bouns_count = { "E9gMeBW0": int }
    ads_bonus_flag = { "qqdr4HlW": int }
    next_day_timer = { "er8Ups6U": int }

@keyjson(key_group = "mebW7mKD", single = True)
class VideoAdsSlotGameInfo:
    game_info = { "C38FmiUn": SlotGameInfo, "write_to_string": True }
    reel_info = { "iW62Scdg": SlotGameReelInfo, "write_to_string": True }
    picture_info = { "rY6j0Jvs": SlotGamePictureInfo, "write_to_string": True }
    game_stand_info = { "tclBMiv2": VideoAdsSlotGameStandInfo, "write_to_string": True }