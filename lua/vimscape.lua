local vimscape = require("vimscape2007")

local typed_letters = {}
local ns = vim.api.nvim_create_namespace("vimscape_keys")
local active = false

local M = {}

M.setup = function(opts)
	print("Options: ", opts)
end

local record_keys = function(key)
	-- Return if we're not actively listening
	if not active then
		return
	end

	-- Ignore insert mode
	local mode = vim.api.nvim_get_mode().mode
	if mode == "i" then
		return
	end

	if #typed_letters >= 10 then
		local string_value = table.concat(typed_letters)
		vimscape.process_batch(string_value)
		typed_letters = {}
	end

	table.insert(typed_letters, key)
end

M.toggle = function()
	active = not active

	if active then
		print("Recording active")
	else
		print("Recording stopped")
		vim.api.nvim_buf_clear_namespace(0, ns, 0, -1)
		typed_letters = {}
	end
end

vim.on_key(record_keys, ns)

return M
