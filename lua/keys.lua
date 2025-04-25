local vimscape = require("vimscape_backend")
local globals = require("globals")
local config = require("config")

---@class Keys
local M = {}

---@return string | nil
M.sanitize_key = function(key)
	local b = key:byte()
	if b <= 126 and b >= 33 then
		return key
	end

	local translated = vim.fn.keytrans(key)

	-- Mouse events
	if translated:match("Left") or translated:match("Mouse") or translated:match("Scroll") then
		return nil
	end

	-- Keybound events show up as this for some reason? Skip 'em
	if translated:match("^<t_..>$") then
		return nil
	end

	-- Ignore Escape
	if translated:match("<Cmd>") then
		return nil
	end

	if translated:match("<CR>") then
		return "|enter|"
	end

	return translated
end

M.record_keys = function(key)
	-- Return if we're not actively listening
	if not globals.active then
		return
	end

	-- Ignore insert mode
	local mode = vim.api.nvim_get_mode().mode
	if mode == "i" then
		return
	end

	local new_key = M.sanitize_key(key)

	if new_key == nil then
		return
	end

	if #globals.typed_letters >= config.batch_size then
		local string_value = table.concat(globals.typed_letters)
		vimscape.process_batch(string_value, config.db_path)
		vim.notify("Processed batch", vim.log.levels.INFO, {})
		globals.typed_letters = {}
	end

	table.insert(globals.typed_letters, new_key)
end

return M
