---@class Globals
---@field get_active function Get active status
---@field set_active function Set active status
---@field get_typed_letters function Get typed letters
---@field set_typed_letters function Set typed letters
---@field clear_typed_letters function Clear typed letters
local M = {}

local state = {
	active = false,
	typed_letters = {},
}

M.get_active = function()
	return state.active
end

M.set_active = function(val)
	state.active = val
end

M.get_typed_letters = function()
	return state.typed_letters
end

M.set_typed_letters = function(val)
	state.typed_letters = val
end

M.clear_typed_letters = function()
	state.typed_letters = {}
end

return M
