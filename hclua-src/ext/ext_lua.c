#include "lua.h"
#include "lstate.h"

extern global_State* lua_getgs(lua_State* L) {
    return L->l_G;
}