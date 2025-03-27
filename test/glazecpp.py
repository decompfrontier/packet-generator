from .tester import Tester

class GlazeCppTester(Tester):
    def run(self):
        from data.net.signalkey import SignalKey
        from data.net.challenge_arena import ChallengeArenaUserInfo
        from data.net.daily_login import DailyLoginRewardsUserInfo
        from data.net.logincampaign import UserLoginCampaignInfo
        from data.net.gme import GmeErrorFlags, GmeErrorCommand, GmeBody, GmeError, GmeHeader, GmeAction
        from data.mst.gatcha import GachaMst, GachaCategory
        from data.mst.npc import NpcMst
        from data.mst.logincampaign import LoginCampaignMst
        from data.mst.town import TownFacilityLvMst

        self.output_file("data/net/signalkey.py", "test/glazecpp/generated/signalkey.hpp", "c++", [ SignalKey ])
        self.output_file("data/net/challenge_arena.py", "test/glazecpp/generated/challenge_arena.hpp", "c++", [ ChallengeArenaUserInfo ])
        self.output_file("data/net/daily_login.py", "test/glazecpp/generated/daily_login.hpp", "c++", [ DailyLoginRewardsUserInfo ])
        self.output_file("data/net/logincampaign.py", "test/glazecpp/generated/logincampaign.hpp", "c++", [ UserLoginCampaignInfo, LoginCampaignMst ])
        self.output_file("data/net/gme.py", "test/glazecpp/generated/gme.hpp", "c++", [ GmeErrorFlags, GmeErrorCommand, GmeBody, GmeError, GmeHeader, GmeAction ])
        self.output_file("data/mst/npc.py", "test/glazecpp/generated/npc.hpp", "c++", [ NpcMst ])
        self.output_file("data/mst/gacha.py", "test/glazecpp/generated/gacha.hpp", "c++", [ GachaMst, GachaCategory ])
        self.output_file("data/mst/town.py", "test/glazecpp/generated/town.hpp", "c++", [ TownFacilityLvMst ])
