from schema import *
from datetime import datetime
from schema import *
from generator import Generator

class DecompGenerator(Generator):
    """This class implements a compatible packet generator for the decompfrontier
    decompilation project.
    
    Challenges on generating for the decomp:
    1. Several properties/mappings might be different from their real-life implementation.
    2. Classes that are generated needs to inject custom functions or properties, some common mechanism
    is needed that doesn't poison the classes too much. Custom functions might be required even to set up
    some properties.
    3. Account for the XOR generation differences that doesn't happend in a real decomp (and they must not!)
    
    How the game actually loads this network JSON?
    
    * Every `keyjson` is mapped inside a super class: GameResponseParser::getResponseObject

    Each of the response are created, for example this is the handling of the BannerInfoMst:

        if ( !strcmp(a2, "Pk5F1vhx") )
        {
            v2 = (MessageResponse *)operator new(0x20uLL);
            BannerInfoMstResponse::BannerInfoMstResponse((BannerInfoMstResponse *)v2);
        }

    The generator would have to first generate something similar based from the mapping
    that were created, a map might be needed to map all this requests to their specific class names.

    Every of this Response classes implements the `BaseResponse` interface, the actual parsing is done
    by implementing the virtual function readParam(int, int, const char*, const char*, bool).

    * All the class objects like BannerInfoMst have their own implementation, which might have special
    properties or custom things, for the sake of completeness the project will not do a 1:1 match of all this classes.
    
    A class like BannerInfoMst consists of different CC_SYNTHETIZE and CC_SYNTHESIZE_XOR or CC_PROPERTY in case of a custom
    getter/setter, the generator has to take in consideration this differences.

    * For array based types, the responses will use something like BannerInfoMstList to store the
    array data, this is another thing where custom operations might and will take effect.

    * The response works by mapping the keys to a specific parsing, either of a single
    class in case of an array or by using their shared reference.

    Array-based parsing example (BannerInfoMst):

        if ( !strcmp(a4, "XuJL4pc5") )
        {
            v26 = v8->m_mst;
            v27 = CommonUtils::StrToInt((CommonUtils *)a5, "XuJL4pc5");
            BannerInfoMst::setDispOrder(v26, v27);
        }

    Creation of the Mst list:

        if ( !a3 )
        {
            if ( !a2 )
            {
            v9 = (BannerInfoMstList *)BannerInfoMstList::shared((BannerInfoMstList *)this);
            BannerInfoMstList::removeAllObjects(v9);
            }
            v10 = (BannerInfoMst *)operator new(0xD0uLL);
            BannerInfoMst::BannerInfoMst(v10);
            this->m_mst = v10;
        }
    
    Push of the Mst list:

        if ( a6 )
        {
        LABEL_72:
            v49 = (BannerInfoMstList *)BannerInfoMstList::shared(v48);
            BannerInfoMstList::addObject(v49, v8->m_mst);
        }

    * The game code makes no distinction between single array and normal arrays 

    * In case the type is not an array, the singleton instance is used (eww)   

    * Gme itself has a custom configuration so it should not be configured as such.

    * Some custom json types like "C38FmiUn" embeds inline JSON parsing with picojson or optionally simdjson,
    this further complicates the parsing of complex objects in a brave frontier context.
        
    
    Current idea:
        - Create a specific format to hold the special metadata changes/injections
        - Parse it as an extra information based from the file name?
        - Iter all properties and see if they should be SYNTHERIZE, XOR or PROPERTY
        - Manually generate the c++ file based from this (potentially dangerous and prone to errors?)
        - Use git with extra caution when reviewing packet generator changes

    """

    def get_start_mark(self, file_name: str) -> str:
        return """
/*
    This file was autogenerated by decompfrontier packet-generator.
    Generation date: {}

    DO NOT MODIFY THIS FILE! MODIFY THE FILE {} INSIDE THE
    PACKET GENERATOR REPOSITORY AND RUN THE GENERATION AGAIN!
*/
#pragma once

#include "PacketHelpers.h"

"""

    def get_end_mark(self) -> str:
        return ""


    def step(self, struct: GeneratorStruct) -> str:
        pass
